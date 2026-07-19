//! Note and Interval Representation
//!
//! 12TET ON TOP

use std::fmt::Debug;

/// Middle C at Concert Pitch
pub const C4: f32 = 261.6;

/// 50c = half a note interval = 1/24th of an octave
pub const FIFTY_CENTS: f32 = 1.0 / 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NoteName {
    Ab,
    A,
    Bb,
    B,
    C,
    Db,
    D,
    Eb,
    E,
    F,
    Gb,
    G,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteKey(NoteName, i8);

impl Debug for NoteKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl NoteName {
    pub const fn number(self) -> i8 {
        match self {
            Self::C => 0,
            Self::Db => 1,
            Self::D => 2,
            Self::Eb => 3,
            Self::E => 4,
            Self::F => 5,
            Self::Gb => 6,
            Self::G => 7,
            Self::Ab => 8,
            Self::A => 9,
            Self::Bb => 10,
            Self::B => 11,
        }
    }
}

impl ToString for NoteName {
    fn to_string(&self) -> String {
        match self {
            Self::C => "C",
            Self::Db => "Db",
            Self::D => "D",
            Self::Eb => "Eb",
            Self::E => "E",
            Self::F => "F",
            Self::Gb => "Gb",
            Self::G => "G",
            Self::Ab => "Ab",
            Self::A => "A",
            Self::Bb => "Bb",
            Self::B => "B",
        }
        .to_string()
    }
}

impl ToString for NoteKey {
    fn to_string(&self) -> String {
        format!("{}{}", self.0.to_string(), self.1)
    }
}

pub struct Interval(pub i8);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Note {
    frequency: f32,
    nearest_note: NoteName,
    octave: i8,
    cents_off: i8,
}

impl Note {
    pub const CONCERT_PITCH: Note = Note {
        frequency: 440.0,
        nearest_note: NoteName::A,
        octave: 4,
        cents_off: 0,
    };

    pub fn key(&self) -> NoteKey {
        NoteKey(self.nearest_note, self.octave)
    }

    pub fn new(frequency: f32) -> Self {
        // Integer part is octave number, decimal part is note + cents
        let log_frequency = (frequency / C4).log2() + 4.0;

        // Cent offset means flat C's can be registered in that octave
        // TODO: Safe cast instead, though I'd be impressed more than anything if someone had a note >= the 256th octave
        // NOTE: I'm not sure a stable safe cast exists, though in all fairness, I've only spent a few minutes looking
        let octave = (log_frequency + FIFTY_CENTS).floor() as i8;

        let note_number =
            ((log_frequency + FIFTY_CENTS) - (log_frequency + FIFTY_CENTS).floor()) * 12.0;
        let nearest_note = match note_number.floor() as i8 {
            0 => NoteName::C,
            1 => NoteName::Db,
            2 => NoteName::D,
            3 => NoteName::Eb,
            4 => NoteName::E,
            5 => NoteName::F,
            6 => NoteName::Gb,
            7 => NoteName::G,
            8 => NoteName::Ab,
            9 => NoteName::A,
            10 => NoteName::Bb,
            11 => NoteName::B,
            _ => panic!("The world is ending; this should not be possible"),
        };

        let cents_off = (((note_number % 1.0) - 0.5) * 100.0).floor() as i8;

        Self {
            frequency,
            octave,
            nearest_note,
            cents_off,
        }
    }

    pub fn interval_to(&self, other: &Note) -> Interval {
        Interval(other.semitone_number() - self.semitone_number())
    }

    fn semitone_number(&self) -> i8 {
        self.octave * 12 + self.nearest_note.number()
    }
}
