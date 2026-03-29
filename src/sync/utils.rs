use alloc::collections::VecDeque;
use alloc::sync::Arc;

use crate::thread::Thread;

// returns: the index of the chosen thread, choose by priority.
pub fn get_thread_index_by_iter<'a, T, I, F>(iter: I, get_thread: F) -> Option<usize>
where
    I: IntoIterator<Item = &'a T>,
    F: Fn(&T) -> &Arc<Thread>,
    T: 'a,
{
    let mut max_pri: i32 = i32::MIN;
    let mut elem_index: Option<usize> = None;
    for (i, item) in iter.into_iter().enumerate() {
        let pri = get_thread(item).get_priority() as i32;
        if pri > max_pri {
            max_pri = pri;
            elem_index = Some(i);
        }
    }
    elem_index
}

pub fn get_thread_index_iter<'a, I>(iter: I) -> Option<usize>
where
    I: IntoIterator<Item = &'a Arc<Thread>>,
{
    get_thread_index_by_iter(iter, |t| t)
}

pub fn get_thread_index_by<T, F>(queue: &VecDeque<T>, get_thread: F) -> Option<usize>
where
    F: Fn(&T) -> &Arc<Thread>,
{
    get_thread_index_by_iter(queue.iter(), get_thread)
}

pub fn get_thread_index(queue: &VecDeque<Arc<Thread>>) -> Option<usize> {
    get_thread_index_by(queue, |thread| thread)
}