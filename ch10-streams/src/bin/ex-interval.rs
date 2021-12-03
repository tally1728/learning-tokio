use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};
use tokio_stream::Stream;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    // Interval Stream from Scratch
    try_interval().await;

    // Interval Stream with stream! macro
    try_interval_with_macro().await;
}

////////////////////////////////////////////////////////////
// Interval Stream from Scratch
async fn try_interval() {
    let stream = Interval {
        rem: 3,
        delay: Delay {
            when: Instant::now() + Duration::from_millis(10),
            waker: None,
        },
    };
    tokio::pin!(stream);

    while let Some(_) = stream.next().await {
        println!("I've got () from the Stream!");
    }
}

struct Interval {
    rem: usize,
    delay: Delay,
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.rem == 0 {
            // No more delays
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.delay).poll(cx) {
            Poll::Ready(_) => {
                let when = self.delay.when + Duration::from_millis(10);
                self.delay = Delay { when, waker: None };
                self.rem -= 1;
                Poll::Ready(Some(()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

////////////////////////////////////////////////////////////
// Interval Stream with stream! macro
async fn try_interval_with_macro() {
    let stream = async_stream::stream! {
        let mut when = Instant::now();
        for _ in 0..3 {
            let delay = Delay {
                when,
                waker: None,
            };
            delay.await;
            yield ();
            when += Duration::from_millis(10);
        }
    };

    tokio::pin!(stream);

    while let Some(_) = stream.next().await {
        println!("I've got () from the Stream!");
    }
}

// Delay Future from 'Async in depth'
struct Delay {
    when: Instant,
    // This Some when we have spawned a thread, and None otherwise.
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();

            if !waker.will_wake(cx.waker()) {
                println!("Delay: update a waker");
                *waker = cx.waker().clone();
            }
        } else {
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());

            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                println!("Delay: Wake up!");
                let waker = waker.lock().unwrap();
                waker.wake_by_ref();
            });
            println!("Delay: Spawn a notification thread");
        }

        if Instant::now() >= self.when {
            println!("Delay: Ready");
            Poll::Ready(())
        } else {
            println!("Delay: Pending...");
            Poll::Pending
        }
    }
}
