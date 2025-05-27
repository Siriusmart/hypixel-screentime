use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::Write,
    path::PathBuf,
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub keys: Vec<String>,
    pub interval: u64,
    pub discord_token: String,
    pub broadcast_channels: HashMap<u64, BroadcastChannel>,
    pub port: u16,
    pub merge: u64,
    pub expire: u64,
    pub users: Vec<Identifier>,
}

#[derive(Serialize, Deserialize)]
pub struct BroadcastChannel {
    pub online: String,
    pub resumed: String,
    pub offline: String,
    pub users: HashSet<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Identifier {
    pub uuid: String,
    pub name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            interval: 60,
            discord_token: String::new(),
            broadcast_channels: HashMap::new(),
            port: 8080,
            merge: 60,
            expire: 70,
            users: Vec::new(),
        }
    }
}

impl Config {
    pub fn get() -> &'static Config {
        static CONFIG: OnceLock<Config> = OnceLock::new();

        CONFIG.get_or_init(Self::init)
    }

    pub fn init() -> Config {
        let path = PathBuf::from(env::var("CONFIG").expect("missing ENV `CONFIG`"));

        fs::create_dir_all(&path).unwrap();

        let path = path.join("master.json");

        if !fs::exists(&path).unwrap() {
            fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)
                .unwrap()
                .write_all(
                    serde_json::to_vec_pretty(&Config::default())
                        .unwrap()
                        .as_slice(),
                )
                .unwrap();
        }

        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap()
    }
}
