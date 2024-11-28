use crate::task::{Task, TaskId};
use crate::waker::MpscWaker;
use futures::task::Poll;
use futures::Future;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Runtime {
    // Unique task id counter
    task_id_counter: Rc<Cell<TaskId>>,
    // Task park
    // Task id is used to get the task from the task park
    task_park: Rc<RefCell<BTreeMap<TaskId, Task>>>,
    // Ready task sender
    ready_task_tx: Sender<TaskId>,
    // Ready task receiver
    // When the waker is called, the task id is pushed to the run queue
    // Task id is used to get the task from the task park
    // The task is polled and if it is pending, it is pushed back to the task park
    // If it is ready, it is dropped
    ready_task_rx: Arc<Mutex<Receiver<TaskId>>>,
}

impl Runtime {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            task_id_counter: Rc::new(Cell::new(0)),
            task_park: Rc::new(RefCell::new(BTreeMap::new())),
            ready_task_tx: tx,
            ready_task_rx: Arc::new(Mutex::new(rx)),
        }
    }

    // Spawn a task
    // The task is polled and if it is pending, it is pushed to the task park
    // If it is ready, it is dropped
    pub fn spawn(&self, f: impl Future<Output = ()> + 'static) {
        let task_id = self.task_id_counter.get();
        self.task_id_counter.set(task_id + 1);
        let waker = MpscWaker::waker(task_id, self.ready_task_tx.clone());
        let mut task = Task::new(f);
        // poll the task
        match task.poll(waker) {
            Poll::Ready(()) => {
                // task is dropped
            }
            Poll::Pending => {
                // task is parked
                self.task_park.borrow_mut().insert(task_id, task);
            }
        }
    }

    // Run the runtime
    // The runtime is a event loop that polls the tasks
    pub fn run(&self, f: impl Future<Output = ()> + 'static) {
        // spawn the main task
        self.spawn(f);
        loop {
            // receive the ready task id
            let task_id = self.ready_task_rx.lock().unwrap().recv().unwrap();
            // get the task from the task park
            let mut task = self.task_park.borrow_mut().remove(&task_id).unwrap();
            // pass the waker (=context) to the task and poll it
            let waker = MpscWaker::waker(task_id, self.ready_task_tx.clone());
            match task.poll(waker) {
                // advance the task and if it is pending, push it back to the task park
                Poll::Pending => {
                    self.task_park.borrow_mut().insert(task_id, task);
                }
                // task is ready, so drop it
                Poll::Ready(()) => {}
            }
            // break the loop if the task park is empty, i.e. all tasks are ready
            if self.task_park.borrow_mut().is_empty() {
                break;
            }
        }
    }
}
