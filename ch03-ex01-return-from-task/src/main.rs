#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async { "return value" });

 println!("Return from a Tokio Task: {}", handle.await.unwrap());
}
