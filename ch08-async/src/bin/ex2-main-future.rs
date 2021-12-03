use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

// Delay Future
struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Delay: Ready");
            Poll::Ready("done")
        } else {
            // Ignore this line for now.
            cx.waker().wake_by_ref();
            println!("Delay: Pending...");
            Poll::Pending
        }
    }
}

// Main Future
enum MainFuture {
    // Initialized, never polled
    State0,
    // Waiting on `Delay`, i.e. the `future.await` line.
    State1(Delay),
    // The future has completed.
    Terminated,
}

impl Future for MainFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        loop {
            match *self {
                MainFuture::State0 => {
                    println!("MainFuture: State0");
                    let when = Instant::now() + Duration::from_millis(10);
                    let future = Delay { when };
                    *self = MainFuture::State1(future);
                }
                MainFuture::State1(ref mut future) => {
                    println!("MainFuture: State1(polling Delay)");
                    match Pin::new(future).poll(cx) {
                        Poll::Ready(out) => {
                            assert_eq!(out, "done");
                            *self = MainFuture::Terminated;
                            return Poll::Ready(());
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
                MainFuture::Terminated => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let future = MainFuture::State0;
    future.await;
}
