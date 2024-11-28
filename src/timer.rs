use futures::task::{Context, Poll, Waker};
use futures::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

// Timeout
pub struct Timeout {
    // Timer thread handle
    th: Option<JoinHandle<()>>,
    // Timer state
    state: Arc<Mutex<Poll<()>>>,
    // Waker is used to wake up the task
    waker: Arc<Mutex<Option<Waker>>>,
}

// Timeout is a Future that will be ready after the specified duration
impl Timeout {
    // Set a timeout
    pub fn set(duration: Duration) -> Self {
        let waker = Arc::new(Mutex::new(None::<Waker>));
        let state = Arc::new(Mutex::new(Poll::Pending));
        let w = waker.clone();
        let s = state.clone();
        let th = std::thread::spawn(move || {
            // wait...
            std::thread::sleep(duration);
            // wait is over
            let mut state = s.lock().unwrap();
            *state = Poll::Ready(());
            // call waker to wake up the task
            // waker is store when poll is called
            if let Some(waker) = &*w.lock().unwrap() {
                waker.wake_by_ref()
            }
        });
        Self {
            th: Some(th),
            state,
            waker,
        }
    }
}

impl Future for Timeout {
    type Output = ();
    // borrow waker at called poll first time
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        *self.waker.lock().unwrap() = Some(ctx.waker().clone());
        *self.state.lock().unwrap()
    }
}

impl Drop for Timeout {
    fn drop(&mut self) {
        self.th.take().unwrap().join().unwrap();
    }
}
