fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Subvox",
        native_options,
        Box::new(|cc| Ok(Box::new(subvox_frontend::App::new(cc)?))),
    )?;

    Ok(())
}
