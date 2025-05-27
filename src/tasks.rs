use std::time::Duration;

use crate::{Config, STORAGE_COPY, Storage};

pub async fn fetch() {
    let config = Config::get();
    let mut key_index = (chrono::Utc::now().timestamp_micros() % config.keys.len() as i64) as usize;

    loop {
        let cloned = Storage::get().lock().await.clone();
        unsafe {
            if STORAGE_COPY.get().is_some() {
                *STORAGE_COPY.get_mut().unwrap() = cloned;
            } else {
                let _ = STORAGE_COPY.set(cloned);
            }
        }
        let now = chrono::Utc::now().timestamp() as u64;
        let sleep = (Storage::get().lock().await.last_fetch + config.interval).saturating_sub(now);
        tokio::time::sleep(Duration::from_secs(sleep)).await;
        Storage::fetch(&mut *Storage::get().lock().await, &config.keys[key_index]).await;
        let now = chrono::Utc::now().timestamp() as u64;
        Storage::get().lock().await.last_fetch = now;

        key_index = (key_index + 1) % config.keys.len();
    }
}
