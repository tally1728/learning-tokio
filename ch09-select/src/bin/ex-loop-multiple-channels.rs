use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx1, mut rx1) = mpsc::channel(128);
    let (tx2, mut rx2) = mpsc::channel(128);
    let (tx3, mut rx3) = mpsc::channel(128);

    // send 0..5 on each channel
    for tx in [tx1, tx2, tx3].into_iter() {
        tokio::spawn(async move {
            for v in 0..5 {
                tx.send(v).await.unwrap();
            }
        });
    }

    loop {
        tokio::select! {
            Some(v) = rx1.recv() => { println!("ch1: {}", v); }
            Some(v) = rx2.recv() => { println!("ch2: {}", v); }
            Some(v) = rx3.recv() => { println!("ch3: {}", v); }
            else => { break; }
        }
    }
}
