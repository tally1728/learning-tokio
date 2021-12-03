use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

// println endlessly
async fn endless() {
    loop {
        println!("endless...");
        sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();

    // send a stop signal via the channel after 100ms
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;

        tx.send(()).unwrap();
    });

    // stop endless by the signal
    tokio::select! {
        _ = endless() => {}
        _ = rx => {}
    }
}
