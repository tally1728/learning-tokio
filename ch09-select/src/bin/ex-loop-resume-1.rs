use tokio::time::{sleep, Duration, Instant};

async fn action() {
    println!("action begins...");

    for v in 0..5 {
        sleep(Duration::from_millis(100)).await;
        println!("action: {}", v);
    }

    sleep(Duration::from_millis(100)).await;
    println!("action ends");
}

async fn restart_op() {
    let time = Instant::now();

    for _ in 0..5 {
        tokio::select! {
            _ = action() => {
                println!("{:03}ms - select: op is done", time.elapsed().as_millis());
                break;
            }
            _ = sleep(Duration::from_millis(200)) => {
                println!("{:03}ms - select: break", time.elapsed().as_millis());
            }
        }

        println!();
    }
}

async fn resume_op() {
    let op = action();
    tokio::pin!(op);

    let time = Instant::now();

    loop {
        tokio::select! {
            _ = &mut op => {
                println!("{:03}ms - select: op is done", time.elapsed().as_millis());
                break;
            }
            _ = sleep(Duration::from_millis(200)) => {
                println!("{:03}ms - select: break", time.elapsed().as_millis());
            }
        }

        println!();
    }
}

#[tokio::main]
async fn main() {
    println!("#### action(): count up ####");
    action().await;

    println!("\n#### loop with breaks: restart_op() ####");
    restart_op().await;

    println!("\n#### loop with breaks: resume_op() ####");
    resume_op().await;
}
