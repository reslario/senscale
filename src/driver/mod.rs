mod settings;
mod write;

pub fn set_sens(sens: f64) -> std::io::Result<()> {
    let mut settings = settings::Settings::default();
    settings.set_sens(sens);

    write::write_settings(&mut settings)
}
