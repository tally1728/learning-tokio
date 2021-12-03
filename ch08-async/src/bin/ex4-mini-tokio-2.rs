use crossbeam::channel;
use futures::task::{self, ArcWake};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
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
            // Get a handle to the waker for the current task
            let waker = cx.waker().clone();
            let when = self.when;

            // Spawn a timer thread.
            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                println!("Delay: Wake up!");
                waker.wake();
            });

            println!("Delay: Pending...");
            Poll::Pending
        }
    }
}

// MiniTokio rev.2
type Receiver = channel::Receiver<Arc<Task>>;
type Sender = channel::Sender<Arc<Task>>;

struct Task {
    // The `Mutex` is to make `Task` implement `Sync`. Only
    // one thread accesses `future` at any given time. The
    // `Mutex` is not required for correctness. Real Tokio
    // does not use a mutex here, but real Tokio has
    // more lines of code than can fit in a single tutorial
    // page.
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: Sender,
}

impl Task {
    // Spawns a new task with the given future.
    //
    // Initializes a new Task harness containing the given future and pushes it
    // onto `sender`. The receiver half of the channel will get the task and
    // execute it.
    fn spawn<F>(future: F, sender: &Sender)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });

        sender.send(task).unwrap();
    }

    fn poll(self: Arc<Self>) {
        println!(
            "Task::poll() Begin - Strong Count of Arc<Task>: {} ",
            Arc::strong_count(&self)
        );

        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl from above.
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        // No other thread ever tries to lock the future
        let mut future = self.future.try_lock().unwrap();

        // Poll the future
        let _ = future.as_mut().poll(&mut cx);

        println!(
            "Task::poll() End - Strong Count of Arc<Task>: {} ",
            Arc::strong_count(&self)
        );
    }

    fn schedule(self: &Arc<Task>) {
        self.executor.send(self.clone()).unwrap();
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        println!(
            "Task::wake_byref() Begin - Strong Count of Arc<Task>: {} ",
            Arc::strong_count(arc_self)
        );
        arc_self.schedule();
        println!(
            "Task::wake_byref() End - Strong Count of Arc<Task>: {} ",
            Arc::strong_count(arc_self)
        );
    }
}

struct MiniTokio {
    scheduled: Receiver,
    sender: Sender,
}

impl MiniTokio {
    /// Initialize a new mini-tokio instance.
    fn new() -> MiniTokio {
        let (sender, scheduled) = channel::unbounded();
        MiniTokio { scheduled, sender }
    }

    /// Spawn a future onto the mini-tokio instance.
    ///
    /// The given future is wrapped with the `Task` harness and pushed into the
    /// `scheduled` queue. The future will be executed when `run` is called.
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}

fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };

        let out = future.await;

        assert_eq!(out, "done");
    });

    mini_tokio.run();
}
