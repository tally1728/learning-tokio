use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    println!("server: listen...");

    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("server: a connection is accepted.");

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    // Return value of `Ok(0)` signifies that the remote has
                    // closed
                    Ok(0) => {
                        println!("server: the connection is closed.");
                        return;
                    }
                    Ok(n) => {
                        // Copy the data back to socket
                        if socket.write(&mut buf[..n]).await.is_err() {
                            // Unexpected socket error. There isn't much we can
                            // do here so just stop processing.
                            println!("server: error");
                            return;
                        }
                        println!("server: echo");
                    }
                    Err(_) => {
                        // Unexpected socket error. There isn't much we can do
                        // here so just stop processing.
                        println!("server: error");
                        return;
                    }
                }
            }
        });
    }
}
