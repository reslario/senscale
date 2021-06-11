mod settings;
mod write;

use settings::Settings;

pub struct Driver {
    sensitivity: f64
}

impl Driver {
    pub fn new() -> Driver {
        Driver {
            sensitivity: f64::NAN
        }
    }

    pub fn set_sens(&mut self, sens: f64) -> std::io::Result<()> {
        // this is only meant to prevent setting the sensitivity
        // to the exact same value consecutively, so an exact
        // comparison is fine here
        #[allow(clippy::float_cmp)] 
        if sens != self.sensitivity {
            self.sensitivity = sens;

            let mut settings = Settings::default();
            settings.set_sens(sens);
        
            write::write_settings(&mut settings)
        } else {
            Ok(())
        }
    }
}
