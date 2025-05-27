use hypixel_screentime::{run, tasks};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    unsafe {
        tokio::join!(tasks::fetch(), run());
    }
}
