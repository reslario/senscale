mod settings;
mod write;

pub fn set_sens(sens: f64) -> std::io::Result<()> {
    let mut settings = settings::Settings::default();
    settings.set_sens(sens);

    write::write_settings(&mut settings)
}

pub fn reset()-> std::io::Result<()> {
    write::write_settings(&mut <_>::default())
}
