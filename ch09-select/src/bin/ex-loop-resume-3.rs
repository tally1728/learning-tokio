use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};

async fn action(input: Option<u32>) {
    println!("action begins...");

    if let Some(v) = input {
        for v in 0..v {
            sleep(Duration::from_millis(100)).await;
            println!("action: {}", v);
        }

        sleep(Duration::from_millis(100)).await;
        println!("action ends");
    } else {
        println!("action nothing to do...");
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(128);

    let op = action(None);
    tokio::pin!(op);
    let mut done = false;

    let time = Instant::now();

    // send 1..5 every 300ms
    tokio::spawn(async move {
        for v in 1..5 {
            tx.send(v).await.unwrap();
            sleep(Duration::from_millis(300)).await;
        }
    });

    loop {
        tokio::select! {
            _ = &mut op, if !done  => {
                done = true;
                println!("{:03}ms - select: op is done", time.elapsed().as_millis());
            }
            Some(v) = rx.recv() => {
                println!("{:03}ms - select: recv {}", time.elapsed().as_millis(), v);
                if v % 2 == 0 {
                    op.set(action(Some(v)));
                    done = false;
                    println!("{:03}ms - select: op restart", time.elapsed().as_millis());
                }
            }
            else => { break; }
        }

        println!();
    }
}
