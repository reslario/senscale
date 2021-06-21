mod settings;
mod write;

use {
    settings::Settings,
    std::{
        io,
        fs::File
    }
};

pub struct Driver {
    sensitivity: f64,
    handle: File
}

impl Driver {
    pub fn new() -> io::Result<Driver> {
        File::open(r"\\.\rawaccel")
            .map_err(rawaccel_file_error)
            .map(|handle| Driver {
                sensitivity: f64::NAN,
                handle 
            })
    }

    pub fn set_sens(&mut self, sens: f64) -> io::Result<()> {
        // this is only meant to prevent setting the sensitivity
        // to the exact same value consecutively, so an exact
        // comparison is fine here
        #[allow(clippy::float_cmp)] 
        if sens != self.sensitivity {
            self.sensitivity = sens;

            let mut settings = Settings::default();
            settings.set_sens(sens);
        
            write::write_settings(&self.handle, &mut settings)
        } else {
            Ok(())
        }
    }
}

fn rawaccel_file_error(e: io::Error) -> io::Error {
    if e.kind() == io::ErrorKind::NotFound {
        io::Error::new(e.kind(), "RawAccel driver not installed")
    } else {
        e
    }
}
