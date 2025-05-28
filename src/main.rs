use hypixel_screentime::{run, start_discord, tasks};

#[tokio::main]
async fn main() {
    tokio::join!(tasks::fetch(), run(), start_discord());
}
