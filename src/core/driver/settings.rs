const MAX_DEV_ID_LEN: usize = 200;

#[repr(C)]
pub struct Vec2<T> {
    x: T,
    y: T
}

impl <T> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 { x, y }
    }

    pub fn both(val: T) -> Vec2<T>
    where T: Clone {
        Vec2::new(val.clone(), val)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(unused)]
pub enum AccelMode {
    Linear,
    Classic,
    Natural,
    NaturalGain,
    Power,
    Motivity,
    NoAccel
}

#[repr(C)]
#[allow(unused)]
#[derive(Copy, Clone)]
pub struct AccelArgs {
    offset: f64,
    legacy_offset: bool,
    accel: f64,
    scale: f64,
    limit: f64,
    exponent: f64,
    midpoint: f64,
    weight: f64,
    scale_cap: f64,
    gain_cap: f64,
    speed_cap: f64,
}

#[repr(C)]
#[allow(unused)]
pub struct DomainArgs {
    domain_weights: Vec2<f64>,
    lp_norm: f64,
}

#[repr(C)]
#[allow(unused)]
pub struct Settings {
    degrees_rotation: f64,
    degrees_snap: f64,
    combine_mags: bool,
    modes: Vec2<AccelMode>,
    argsv: Vec2<AccelArgs>,
    sens: Vec2<f64>,
    dir_multipliers: Vec2<f64>,
    domain_args: DomainArgs,
    range_weights: Vec2<f64>,
    time_min: f64,
    device_id: [u16; MAX_DEV_ID_LEN]
}

impl Settings {
    pub fn set_sens(&mut self, sens: f64) {
        self.sens = Vec2::both(sens)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            degrees_rotation: 0.,
            degrees_snap: 0.,
            combine_mags: true,
            modes: Vec2::both(AccelMode::NoAccel),
            argsv: Vec2::both(AccelArgs {
                offset: 0.,
                legacy_offset: false,
                accel: 0.,
                scale: 1.,
                limit: 2.,
                exponent: 2.,
                midpoint: 10.,
                weight: 1.,
                scale_cap: 0.,
                gain_cap: 0.,
                speed_cap: 0.,
            }),
            sens: Vec2::both(1.),
            dir_multipliers: Vec2::both(1.),
            domain_args: DomainArgs {
                domain_weights: Vec2::both(1.),
                lp_norm: 2.,
            },
            range_weights: Vec2::both(1.),
            time_min: 0.1,
            device_id: [0; MAX_DEV_ID_LEN]
        }
    }
}
