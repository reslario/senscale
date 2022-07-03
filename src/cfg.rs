use {
    crate::Result,
    directories::ProjectDirs,
    linked_hash_map::LinkedHashMap,
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        fs::{self, File},
        io::{BufReader, BufWriter, Write},
        path::{Path, PathBuf},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericConfig<E> {
    #[serde(default = "default_sensitivity")]
    pub default_sensitivity: f64,
    pub processes: E,
}

pub type Config = GenericConfig<HashMap<PathBuf, Entry>>;
pub type EditableConfig = GenericConfig<LinkedHashMap<PathBuf, Entry>>;

impl<E: Default> Default for GenericConfig<E> {
    fn default() -> Self {
        GenericConfig {
            default_sensitivity: default_sensitivity(),
            processes: <_>::default(),
        }
    }
}

pub const fn default_sensitivity() -> f64 {
    1.0
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConfigEntry {
    Short(f64),
    Long(Entry),
}

impl From<ConfigEntry> for Entry {
    fn from(entry: ConfigEntry) -> Self {
        match entry {
            ConfigEntry::Long(entry) => entry,
            ConfigEntry::Short(sensitivity) => Entry {
                sensitivity,
                only_if_cursor_hidden: <_>::default(),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "ConfigEntry")]
pub struct Entry {
    pub sensitivity: f64,
    #[serde(default)]
    pub only_if_cursor_hidden: bool,
}

impl Entry {
    fn can_be_short(&self) -> bool {
        self.only_if_cursor_hidden == <_>::default()
    }
}

impl Serialize for Entry {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.can_be_short() {
            ConfigEntry::Short(self.sensitivity).serialize(serializer)
        } else {
            #[derive(Serialize)]
            struct EntryProxy {
                sensitivity: f64,
                only_if_cursor_hidden: bool,
            }

            let proxy = EntryProxy {
                sensitivity: self.sensitivity,
                only_if_cursor_hidden: self.only_if_cursor_hidden,
            };

            proxy.serialize(serializer)
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            sensitivity: default_sensitivity(),
            only_if_cursor_hidden: <_>::default(),
        }
    }
}

pub fn read_config() -> Result<Config> {
    let file = config_dir()?.file();

    if file.exists() {
        let yaml = File::open(file)?;

        serde_yaml::from_reader(BufReader::new(yaml)).map_err(<_>::into)
    } else {
        create_config(file)
    }
}

pub fn create_config(file: impl AsRef<Path>) -> Result<Config> {
    if let Some(dir) = file.as_ref().parent() {
        fs::create_dir_all(dir)?
    }

    let config = Config::default();
    write_config(&config, BufWriter::new(File::create(file)?))?;
    Ok(config)
}

const CFG_HEADER: &str = include_str!("./cfg_header.yaml");

pub fn write_config<E: Serialize>(config: &GenericConfig<E>, mut writer: impl Write) -> Result<()> {
    writer.write_all(CFG_HEADER.as_bytes())?;

    serde_yaml::to_writer(writer, config)?;

    Ok(())
}

pub fn config_dir() -> Result<ConfigDir> {
    let dirs = ProjectDirs::from("io.github", "reslario", "senscale")
        .ok_or("couldn't get program directories")?;

    Ok(ConfigDir { dirs })
}

pub struct ConfigDir {
    dirs: ProjectDirs,
}

impl std::ops::Deref for ConfigDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.dirs.config_dir()
    }
}

impl ConfigDir {
    pub fn file(&self) -> PathBuf {
        self.join("config.yaml")
    }
}
