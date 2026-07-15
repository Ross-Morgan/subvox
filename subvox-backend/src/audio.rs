use std::{fs::File, ops::Range, path::Path};

use bytemuck::try_cast_slice;
use memmap2::{Mmap, MmapOptions};

// Header magic stuff
const RIFF: &[u8; 4] = b"RIFF";
const WAVE: &[u8; 4] = b"WAVE";
const FMT_: &[u8; 4] = b"fmt ";
const DATA: &[u8; 4] = b"data";

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SampleFormat {
    Pcm,
    Float,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WavFormat {
    pub format: SampleFormat,
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub block_align: u16,
}

pub struct WavFile {
    mmap: Mmap,
    format: WavFormat,
    data_range: Range<usize>,
}

impl WavFile {
    pub fn format(&self) -> WavFormat {
        self.format
    }

    #[inline]
    pub fn data_bytes(&self) -> &[u8] {
        &self.mmap[self.data_range.clone()]
    }

    // Number of frames
    pub fn duration_frames(&self) -> u64 {
        let block_align = u64::from(self.format.block_align.max(1));
        self.data_bytes().len() as u64 / block_align
    }

    pub fn duration_secs(&self) -> f64 {
        self.duration_frames() as f64 / f64::from(self.format.sample_rate)
    }

    pub fn as_i16_slice(&self) -> Option<&[i16]> {
        if self.format.format != SampleFormat::Pcm || self.format.bits_per_sample != 16 {
            return None;
        }
        try_cast_slice::<u8, i16>(self.data_bytes()).ok()
    }

    pub fn as_f32_slice(&self) -> Option<&[f32]> {
        if self.format.format != SampleFormat::Pcm || self.format.bits_per_sample != 32 {
            return None;
        }
        try_cast_slice::<u8, f32>(self.data_bytes()).ok()
    }

    pub fn to_i16_vec(&self) -> Vec<i16> {
        // Tries zero-copy method first
        if let Some(s) = self.as_i16_slice() {
            return s.to_vec();
        }

        // Fallback path
        self.data_bytes()
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect()
    }

    pub fn to_f32_vec(&self) -> Vec<f32> {
        if let Some(s) = self.as_f32_slice() {
            return s.to_vec();
        }

        match (self.format.format, self.format.bits_per_sample) {
            (SampleFormat::Pcm, 8) => self
                .data_bytes()
                .iter()
                .map(|&b| (f32::from(b) - 128.0) / 128.0)
                .collect(),
            (SampleFormat::Pcm, 16) => self
                .data_bytes()
                .chunks_exact(2)
                .map(|b| f32::from(i16::from_le_bytes([b[0], b[1]])) / f32::from(i16::MAX))
                .collect(),
            (SampleFormat::Pcm, 24) => self
                .data_bytes()
                .chunks_exact(3)
                .map(|b| {
                    let v = i32::from_le_bytes([b[0], b[1], b[2], 0]);
                    ((v << 8) >> 8) as f32 / 8_388_608.0 // 2^23
                })
                .collect(),
            (SampleFormat::Pcm, 32) => self
                .data_bytes()
                .chunks_exact(4)
                .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32 / i32::MAX as f32)
                .collect(),
            (SampleFormat::Float, 32) => self
                .data_bytes()
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect(),
            (fmt, bits) => panic!("unsupported sample encoding: {fmt:?} {bits}-bit"),
        }
    }
}

#[inline]
fn read_u16_le(b: &[u8]) -> u16 {
    u16::from_le_bytes([b[0], b[1]])
}

#[inline]
fn read_u32_le(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// Load a WAV file
///
/// # Panics
///
/// Panics if the file can't be opened/mapped
pub fn load_wav<P: AsRef<Path>>(path: P) -> WavFile {
    let file = File::open(path.as_ref())
        .unwrap_or_else(|e| panic!("failed to open {}: {e}", path.as_ref().display()));

    // SAFETY: UB if read by another process at the same time
    let mmap = unsafe { MmapOptions::new().map(&file) }
        .unwrap_or_else(|e| panic!("failed to mmap {}: {e}", path.as_ref().display()));

    // Hint to OS that the the file will be read sequentially
    // Causes pages to be agressively prefetched
    let _ = mmap.advise(memmap2::Advice::Sequential);

    let file_len = mmap.len();
    assert!(file_len >= 12, "file too small to be a WAV file");

    let (header, _) = mmap.split_at(12);

    assert_eq!(&header[0..4], RIFF, "missing RIFF header");
    assert_eq!(&header[8..12], WAVE, "missing WAVE header");

    let mut offset = 12usize;
    let mut fmt = None;
    let mut data_range = None;

    while offset + 8 <= file_len {
        // TODO: Handle unwraps
        let chunk_header: [u8; 8] = mmap[offset..offset + 8].try_into().unwrap();

        let chunk_id = &chunk_header[0..4];
        let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().unwrap()) as usize;
        let body_start = offset + 8;

        let available = file_len.saturating_sub(body_start);
        let effective_size = chunk_size.min(available);
        let body_end = body_start + effective_size;

        if chunk_id == FMT_ {
            assert!(effective_size >= 16, "fmt chunk too small");
            let fb = &mmap[body_start..body_end];

            let format_tag = u16::from_le_bytes(fb[0..2].try_into().unwrap());
            let channels = u16::from_le_bytes(fb[2..4].try_into().unwrap());
            let sample_rate = u32::from_le_bytes(fb[4..8].try_into().unwrap());
            let block_align = u16::from_le_bytes(fb[12..14].try_into().unwrap());
            let bits_per_sample = u16::from_le_bytes(fb[14..16].try_into().unwrap());

            let resolved_tag = if format_tag == 0xFFFE {
                assert!(effective_size >= 40, "extensible fmt chunk too small");
                u16::from_le_bytes(fb[24..26].try_into().unwrap()) // first two bytes of the sub-format GUID
            } else {
                format_tag
            };

            let format = match resolved_tag {
                1 => SampleFormat::Pcm,
                3 => SampleFormat::Float,
                other => panic!("unsupported WAV format tag: {other}"),
            };

            assert!(channels > 0, "channel count must be non-zero");
            assert!(sample_rate > 0, "sample rate must be non-zero");
            assert!(
                matches!(bits_per_sample, 8 | 16 | 24 | 32),
                "unsupported bits per sample: {bits_per_sample}"
            );

            fmt = Some(WavFormat {
                format,
                channels,
                sample_rate,
                bits_per_sample,
                block_align,
            });
        } else if chunk_id == DATA {
            data_range = Some(body_start..body_end);
        }

        offset = body_start + chunk_size + (chunk_size & 1);
    }

    let format = fmt.expect("invalid WAV file: missing fmt chunk");
    let data_range = data_range.expect("invalid WAV file: missing data chunk");

    let block_align = format.block_align.max(1) as usize;
    assert!(
        (data_range.end - data_range.start).is_multiple_of(block_align),
        "invalid WAV file: data chunk length is not a whole number of frames"
    );

    WavFile {
        mmap,
        format,
        data_range,
    }
}
