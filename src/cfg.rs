use {
    std::fs,
    toml::de,
    crate::Result,
    serde::Deserialize,
    directories::ProjectDirs
};

#[derive(Deserialize)]
#[derive(Default)]
pub struct Config {
    #[serde(default = "default_sensitivity")]
    pub default_sensitivity: f64,
    #[serde(default, rename = "entry")]
    pub entries: Vec<Entry>
}

const fn default_sensitivity() -> f64 {
    1.0 
}

#[derive(Deserialize)]
pub struct Entry {
    pub process: String,
    pub sensitivity: f64,
    #[serde(default)]
    pub only_if_cursor_hidden: bool
}

pub fn read_config() -> Result<Config> {
    let dirs = ProjectDirs::from("io.github", "reslario", "senscale")
        .ok_or("couldn't get program directories")?;
    let dir = dirs.config_dir();
    
    let path = dir.join("config.toml");

    if path.exists() {
        let toml = fs::read_to_string(path)?;
        
        de::from_str(&toml)
            .map_err(<_>::into)
    } else {
        fs::create_dir_all(dir)?;
        fs::write(path, DEFAULT_CFG_FILE)?;
        Ok(<_>::default())
    }
}

macro_rules! lines {
    ($($lit:literal)*) => {
        concat!(
            $( concat!($lit, "\n") ),*
        )
    };
}

const DEFAULT_CFG_FILE: &str = lines!(
    "# format for defining an entry:"
    "#"
    "# [[entry]]"
    "# process = \"example.exe\""
    "# sensitivity = 4.2"
    "# only_if_cursor_hidden = true (optional, will only apply scaling if the cursor is hidden)"
    ""
    "default_sensitivity = 1.0"
);
