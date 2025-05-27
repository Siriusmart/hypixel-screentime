use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    io::Write,
    path::PathBuf,
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex, task::JoinSet};

use crate::Config;

pub static mut STORAGE_COPY: OnceLock<Storage> = OnceLock::new();

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Storage {
    pub last_fetch: u64,
    pub first_fetch: u64,
    pub users: HashMap<String, Vec<Record>>,
}

#[derive(Deserialize)]
pub struct Fetched {
    #[serde(rename = "lastLogin")]
    pub last_login: u64,
    #[serde(rename = "lastLogout")]
    pub last_logout: u64,
}

impl Storage {
    pub fn print_log(user: &str) -> String {
        Self::copy()
            .users
            .get(user)
            .unwrap_or(&Vec::new())
            .iter()
            .rev()
            .map(|record| {
                format!(
                    r#"  <tr>
    <td>{}</td>
    <td>{}</td>
  </tr>"#,
                    {
                        let s = chrono::DateTime::from_timestamp_millis(record.beginning as i64)
                            .unwrap()
                            .to_rfc2822();
                        s[..s.len() - 5].to_string()
                    },
                    if let Some(end) = record.end {
                        let s = chrono::DateTime::from_timestamp_millis(end as i64)
                            .unwrap()
                            .to_rfc2822();
                        s[..s.len() - 5].to_string()
                    } else {
                        "online".to_string()
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn copy() -> &'static Self {
        unsafe { STORAGE_COPY.get().unwrap() }
    }

    pub fn is_online(user: &str) -> bool {
        match Self::copy().users.get(user) {
            Some(val) => !val.is_empty() && val.last().unwrap().end.is_none(),
            None => false,
        }
    }

    pub fn save(&self) {
        let path = PathBuf::from(env::var("CONFIG").expect("missing ENV `CONFIG`"));

        fs::create_dir_all(&path).unwrap();

        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.join("storage.json"))
            .unwrap()
            .write_all(serde_json::to_vec_pretty(self).unwrap().as_slice())
            .unwrap();
    }

    pub fn expire(&mut self) {
        let timeout =
            (chrono::Utc::now().timestamp() as u64 - Config::get().expire * 3600 * 24) * 1000;
        self.users.iter_mut().for_each(|(_, entries)| {
            entries.drain(
                ..entries
                    .iter()
                    .position(|entry| entry.end.is_some_and(|logout| logout < timeout))
                    .unwrap_or(1)
                    - 1,
            );
        });
    }

    pub fn get() -> &'static Mutex<Self> {
        static STORAGE: OnceLock<Mutex<Storage>> = OnceLock::new();

        STORAGE.get_or_init(|| Mutex::new(Self::init()))
    }

    pub fn init() -> Storage {
        let path = PathBuf::from(env::var("CONFIG").expect("missing ENV `CONFIG`"));

        fs::create_dir_all(&path).unwrap();

        let path = path.join("storage.json");

        if !fs::exists(&path).unwrap() {
            fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)
                .unwrap()
                .write_all(
                    serde_json::to_vec_pretty(&Storage::default())
                        .unwrap()
                        .as_slice(),
                )
                .unwrap();
        }

        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap()
    }

    pub fn push_beginning(&mut self, user: String, time: u64) {
        let user = self.users.entry(user).or_default();

        match () {
            _ if user.is_empty() => user.push(Record {
                beginning: time,
                session_begin: time,
                end: None,
            }),
            _ if user.last().unwrap().end.is_none() => {
                *user.last_mut().unwrap() = Record {
                    beginning: time,
                    session_begin: time,
                    end: None,
                }
            }
            _ if user.last().unwrap().beginning == time => {}
            _ if user.last().unwrap().end.unwrap() < time
                && time - user.last().unwrap().end.unwrap() < Config::get().merge * 1000 =>
            {
                user.last_mut().unwrap().end = None;
                user.last_mut().unwrap().session_begin = time;
            }
            _ => user.push(Record {
                beginning: time,
                session_begin: time,
                end: None,
            }),
        }
    }

    pub fn push_logout(&mut self, user: String, time: u64) {
        let user = self.users.entry(user).or_default();

        match () {
            _ if user.is_empty()
                || user.last().unwrap().end.is_some()
                || user.last_mut().unwrap().session_begin > time => {}
            _ => user.last_mut().unwrap().end = Some(time),
        }
    }

    pub async fn fetch_one(uuid: &str, key: &str) -> Result<Fetched, Box<dyn Error>> {
        #[derive(Deserialize)]
        struct PlayerWrapper {
            player: Fetched,
        }
        Ok(serde_json::from_str::<PlayerWrapper>(
            &reqwest::get(format!(
                "https://api.hypixel.net/v2/player?uuid={uuid}&key={key}"
            ))
            .await?
            .text()
            .await?,
        )?
        .player)
    }

    pub async fn fetch(&mut self) {
        let config = Config::get();
        let key = &config.key;

        let mut set = JoinSet::new();

        self.last_fetch = chrono::Utc::now().timestamp() as u64;

        config.users.iter().for_each(|user| {
            let user = user.clone();
            set.spawn(async move {
                match Self::fetch_one(&user.uuid, key).await {
                    Ok(fetched) => return Some((user.name, fetched)),
                    Err(e) => println!(
                        "Failed to fetch user={}, uuid={} - {e}",
                        user.uuid, user.uuid
                    ),
                }

                None
            });
        });

        for (name, fetched) in set
            .join_all()
            .await
            .into_iter()
            .filter_map(std::convert::identity)
        {
            self.push_beginning(name.clone(), fetched.last_login);
            self.push_logout(name.clone(), fetched.last_logout);
        }

        self.expire();
        self.first_fetch = self.first_fetch.max(self.last_fetch - 3600 * 24 * 10);
        self.save();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    pub beginning: u64,
    pub session_begin: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}
