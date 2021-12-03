use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let out = tokio::select! {
        _ = sleep(Duration::from_millis(10)) => 1,
        _ = sleep(Duration::from_millis(10)) => 2,
    };

    println!("output: {}", out);
}
