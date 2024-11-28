use futures::task::{Context, Poll, Waker};
use futures::Future;
use std::pin::Pin;

// Unique task ID
pub type TaskId = usize;

// Task is Boxed Future with no output
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
}

// Task is created with a Future
impl Task {
    // Create a new task with a future
    pub fn new(f: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(f),
        }
    }

    // Poll this task and if it returns Ready, this task is removed from the wait queue
    // If it returns Pending, this task is pushed back to the wait queue
    pub fn poll(&mut self, waker: Waker) -> Poll<()> {
        let mut ctx = Context::from_waker(&waker);
        match Future::poll(self.future.as_mut(), &mut ctx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => Poll::Ready(()),
        }
    }
}
