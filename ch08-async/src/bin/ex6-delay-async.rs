use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::Notify;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // another task counting numbers
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        for i in 0..7 {
            println!("another task... {}", i);
            interval.tick().await;
        }
    });

    println!("delay w/ tokio::sync::Notify...");
    dela(Duration::from_secs(3)).await;
    println!("delay w/ tokio::sync::Notify... Done!");

    println!("tokio::time::sleep...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("tokio::time::sleep... Done!");
}

async fn dela(dur: Duration) {
    let when = Instant::now() + dur;
    let notify = Arc::new(Notify::new());
    let notify2 = Arc::clone(&notify);

    // WRONG -> tokio::spawn(async move {
    thread::spawn(move || {
        let now = Instant::now();

        if now < when {
            thread::sleep(when - now);
        }

        notify2.notify_one();
    });

    notify.notified().await;
}
