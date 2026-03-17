use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::sync::atomic::Ordering::SeqCst;

use crate::thread::Thread;

pub fn get_thread_index_by<T, F>(queue: &VecDeque<T>, get_thread: F) -> Option<usize>
where
    F: Fn(&T) -> &Arc<Thread>,
{
    let mut max_pri: i32 = i32::MIN;
    let mut elem_index: Option<usize> = None;
    for (i, item) in queue.iter().enumerate() {
        let pri = get_thread(item).priority.load(SeqCst) as i32;
        if pri > max_pri {
            max_pri = pri;
            elem_index = Some(i);
        }
    }
    elem_index
}

pub fn get_thread_index(queue: &VecDeque<Arc<Thread>>) -> Option<usize> {
    get_thread_index_by(queue, |thread| thread)
}