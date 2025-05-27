use hypixel_screentime::{run, start_discord, tasks};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    unsafe {
        tokio::join!(tasks::fetch(), run(), start_discord());
    }
}
