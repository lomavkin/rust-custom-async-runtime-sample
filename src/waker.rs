use crate::task::TaskId;
use futures::task::ArcWake;
use futures::task::Waker;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MpscWaker(TaskId, Arc<Mutex<Sender<TaskId>>>);

impl MpscWaker {
    pub fn waker(task_id: TaskId, tx: Sender<TaskId>) -> Waker {
        futures::task::waker(Arc::new(MpscWaker(task_id, Arc::new(Mutex::new(tx)))))
    }
}
impl ArcWake for MpscWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let MpscWaker(task_id, ref tx) = **arc_self;
        tx.lock().unwrap().send(task_id).unwrap();
    }
}
