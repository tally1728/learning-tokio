#[tokio::main]
async fn main() {
    let v = vec![0, 1, 2];

    tokio::spawn(async {
        println!("Here's a vec: {:?}", v);
    });
}
