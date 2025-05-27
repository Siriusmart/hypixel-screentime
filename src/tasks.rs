use std::{sync::atomic::AtomicU64, time::Duration};

use tokio::{sync::Mutex, task};

use crate::{Config, STORAGE_COPY, Storage};

static mut last_save: u64 = 0;

pub async fn fetch() {
    let config = Config::get();

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
        dbg!(sleep);
        tokio::time::sleep(Duration::from_secs(sleep)).await;
        Storage::fetch(&mut *Storage::get().lock().await).await;
        let now = chrono::Utc::now().timestamp() as u64;
        Storage::get().lock().await.last_fetch = now;
    }
}
