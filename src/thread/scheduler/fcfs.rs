use alloc::collections::VecDeque;
use alloc::sync::Arc;

use crate::sync::utils::get_thread_index_iter;
use crate::thread::{current, Schedule, Status, Thread};

/// FIFO scheduler.
#[derive(Default)]
pub struct Fcfs(VecDeque<Arc<Thread>>);

impl Schedule for Fcfs {

    fn register(&mut self, thread: Arc<Thread>) {
        self.0.push_back(thread);
    }

    fn schedule(&mut self) -> Option<Arc<Thread>> {
        let idx = get_thread_index_iter(self.0.iter())?;
        let highest_pri = self.0[idx].get_priority();
        let cur = current();
        let cur_pri = cur.get_priority();
        if highest_pri < cur_pri && cur.status() == Status::Running {
            return None;
        }
        self.0.remove(idx)
    }

}