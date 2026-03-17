use alloc::collections::VecDeque;
use alloc::sync::Arc;

use crate::thread::{current, Schedule, Status, Thread};
use core::sync::atomic::Ordering::SeqCst as order;

/// FIFO scheduler.
#[derive(Default)]
pub struct Fcfs(VecDeque<(i32, VecDeque<Arc<Thread>>)>);

impl Schedule for Fcfs {
    fn register(&mut self, thread: Arc<Thread>) {
        let pri = thread.priority.load(order).cast_signed();
        let mut insert_index = 0;

        for (i, solo) in self.0.iter_mut().enumerate() {
            if solo.0 == pri {
                solo.1.push_front(thread);
                return;
            }
            if solo.0 < pri {
                insert_index = i + 1;
            }
        }

        let mut new_queue = VecDeque::new();
        new_queue.push_front(thread);
        self.0.insert(insert_index, (pri, new_queue));
    }

    fn schedule(&mut self) -> Option<Arc<Thread>> {
        let (highest_pri, _) = self.0.back()?;
        // Keep current running only when it has strictly higher priority than
        // every ready thread. Equal priority uses RR by switching to ready peer.
        let cur = current();
        let cur_pri = cur.priority.load(order).cast_signed();
        if *highest_pri < cur_pri && cur.status() == Status::Running {
            return None;
        }

        let last = self.0.back_mut().expect("checked by back() above");
        let next = last.1.pop_back();
        if last.1.is_empty() {
            self.0.pop_back();
        }

        next
    }
}