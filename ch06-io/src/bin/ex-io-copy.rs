use tokio::io;

#[tokio::main]
async fn main() {
    let orig = b"hello";
    let mut reader = &orig[..];
    let mut writer = Vec::new();

    println!("### Before io::copy() ###");
    println!("orig: {:?}", orig);
    println!("reader: {:?}", reader);
    println!("writer: {:?}\n", writer);

    io::copy(&mut reader, &mut writer).await.unwrap();

    println!("### After io::copy() ###");
    println!("orig: {:?}", orig);
    println!("reader: {:?}", reader);
    println!("writer: {:?}", writer);
}
