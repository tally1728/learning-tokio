#[tokio::main]
async fn main() {
    // error in the async expression
    error_async_expression().await;

    // error in the handler
    if error_handler().await.is_err() {
        println!("handle a parse error!");
    }
}

// error in the async expression
async fn error_async_expression() {
    tokio::select! {
        res = async {
            println!{"an error will occur in the async expression!"};
            // propagates the error out of the async expression
            "o".parse::<u8>()?;
            // Help the rust type inferencer out
            Ok::<_, std::num::ParseIntError>(())
        } => {
            // handling the error fron the async expression
            if res.is_err() {
                println!("handle a parse error!");
            }
        }
    }
}

// error in the handler
async fn error_handler() -> Result<(), std::num::ParseIntError> {
    tokio::select! {
        _ = async {
            println!{"an error will occur in my handler!"};
        } => {
            // propagates the error out of the function
            "o".parse::<u8>()?;
        }
    };

    Ok(())
}
