use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    let mut out = String::new();

    tokio::spawn(async move {
        if tx1.send(1).is_err() {
            println!("tx1 error");
        }
    });

    tokio::spawn(async move {
        if tx2.send(2).is_err() {
            println!("tx2 error");
        }
    });

    tokio::select! {
        _ = rx1 => {
            out.push_str("rx1 completed");
        }
        _ = rx2 => {
            out.push_str("rx2 completed");
        }
    }

    println!("out: {}", out);
}
