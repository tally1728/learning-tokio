use std::time::Instant;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // tokio::spwan
    let start = Instant::now();
    for _ in 0..1000000 {
        let h = tokio::spawn(async move {});
        h.await.unwrap();
    }
    let duration = start.elapsed();
    println!("tokio::spwan {:?}", duration);

    // std::thread::spwan
    let start = Instant::now();
    for _ in 0..1000000 {
        let h = std::thread::spawn(move || {});
        h.join().unwrap();
    }
    let duration = start.elapsed();
    println!("std::thread::spwan {:?}", duration);
}
