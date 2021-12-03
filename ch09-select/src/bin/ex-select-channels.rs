use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // wait for messages from 2 channels
    // and select only earlier one
    select_earlier_message().await;

    // cancel sending operation
    // if the channel is closed
    operation_w_cancellation().await;
}

async fn select_earlier_message() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    tokio::spawn(async move {
        tx1.send("one").unwrap();
    });

    tokio::spawn(async move {
        tx2.send("two").unwrap();
    });

    tokio::select! {
        val = rx1 => { println!("rx1 completed first with {:?}", val); }
        val = rx2 => { println!("rx2 completed first with {:?}", val); }
    }
}

async fn operation_w_cancellation() {
    let (mut tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    tokio::spawn(async move {
        // Select on the operation and the oneshot's
        // `closed()` notification.
        tokio::select! {
            _ = sleep(Duration::from_millis(100)) => {
                tx1.send("one").unwrap();
            }
            // wait for closing tx1
            _ = tx1.closed() => {
                // the operation is canceled, the
                // task completes and `tx1` is dropped.
                println!("rx1 is dropped");
            }
        }
    });

    tokio::spawn(async move {
        //sleep(Duration::from_millis(200)).await;
        tx2.send("two").unwrap();
    });

    tokio::select! {
        val = rx1 => { println!("rx1 completed first with {:?}", val); }
        val = rx2 => { println!("rx2 completed first with {:?}", val); }
    }
}
