# Lab 1: Scheduling

---

## Information

Name: Jia Shengyuan

Email: [jia_shengyuan@stu.pku.edu.cn](jia_shengyuan@stu.pku.edu.cn)

> Please cite any forms of information source that you have consulted during finishing your assignment, except the TacOS documentation, course slides, and course staff.

GPT-5.3-Codex, Gemini 3.1 Pro

> With any comments that may help TAs to evaluate your work better, please leave them here

## Alarm Clock

### Data Structures

> A1: Copy here the **declaration** of every new or modified struct, enum type, and global variable. State the purpose of each within 30 words.

### Algorithms

> A2: Briefly describe what happens in `sleep()` and the timer interrupt handler.

`sleep` 被调用时，会先屏蔽中断，之后将当前线程加入一个等待被唤醒的队列中，并阻塞，然后切换到别的线程。在计时器中断发生时，会检测等待唤醒的队列，并唤醒那些到了唤醒时间的线程。

> A3: What are your efforts to minimize the amount of time spent in the timer interrupt handler?

将等待被唤醒的队列使用 `BTreeMap` 实现，这样插入和删除就均只需 $O(\log n)$ 时间。

### Synchronization

> A4: How are race conditions avoided when `sleep()` is being called concurrently?

在维护睡眠线程的 `BTreeMap` 外套一层 `Mutex`。

> A5: How are race conditions avoided when a timer interrupt occurs during a call to `sleep()`?

在 `sleep()` 内暂时屏蔽了 timer interrupt。

## Priority Scheduling

### Data Structures

> B1: Copy here the **declaration** of every new or modified struct, enum type, and global variable. State the purpose of each within 30 words.

```rust
pub struct Thread {
    tid: isize,
    name: &'static str,
    stack: usize,
    status: Mutex<Status>,
    context: Mutex<Context>,
    locks_held: Mutex<Vec<usize>>,
    lock_waiting: Mutex<Option<usize>>,
    priority: AtomicU32,
    pub userproc: Option<UserProc>,
    pub pagetable: Option<Mutex<PageTable>>,
}
```

`Thread` 类中新增加了对自己持有和自己等待的锁的维护，以方便 priority donation。

```rust
pub struct Condvar(RefCell<VecDeque<(Arc<Semaphore>, Arc<Thread>)>>);
```

`Condvar` 类额外保存自己阻塞了的线程，以实现条件满足时能够按优先级调度。

> B2: Explain the data structure that tracks priority donation. Clarify your answer with any forms of diagram (e.g., the ASCII art).

`Thread -> thread.locks_held -> Sleep -> sleep.inner -> Semaphore -> semaphore.waiters -> Thread(recursed)`

### Algorithms

> B3: How do you ensure that the highest priority thread waiting for a lock, semaphore, or condition variable wakes up first?

实现了一个工具类方法，传入一个关于 `Thread` 的迭代器，返回最高优先级 `Thread` 的下标。之后每次需要时都通过这里获取最高优先级的进程即可。

> B4: Describe the sequence of events when a thread tries to acquire a lock. How is nested donation handled?

首先关闭中断，并记录当前线程。对于要获取的锁，如果已经被其他线程持有，就在当前线程中记录正在等候这把锁，以及在持有锁的线程中记录这个锁正在被当前线程等。之后进行信号量的 `down()` 操作，若没有人持有锁则拿到锁，否则进入 `waiters` 列表并阻塞。等到阻塞结束后（或者压根没阻塞），当前线程就拿到锁，之后更新锁相关的记录信息（比如将当前锁的拥有者改为当前线程），之后继续正常运行即可。

Nested donation: 在线程中做了一个修改，把 `priority` 字段改为私有，并新增 `get_priority()` 接口供外部获取优先级。调用此接口时，会遍历所有正在等待当前线程的 `Sleep`，并通过这些 `Sleep` 实例最终找到正在等待当前线程的其他线程，之后递归调用这些线程的 `get_priority` 并最终取 max 即可。

> B5: Describe the sequence of events when a lock, which a higher-priority thread is waiting for, is released.

锁被释放时，会释放信号量，之后在等待列表中寻找优先级最高的一个线程，并将其唤醒。由于等待线程的优先级较高，会立即调用 `schedule()` 将控制权移交。

### Synchronization

> B6: Describe a potential race in `thread::set_priority()` and explain how your implementation avoids it. Can you use a lock to avoid this race?

潜在竞争：在修改完线程优先级后，调用 `schedule()` 前的这段时间内，如果恰好发生调度，可能会出现新的调度结果实质上是基于旧的优先级的情况。实现中，写新优先级的过程使用原子存储，写后立即进行 `schedule()`，并且进行所有调度时都使用 `get_priority()` 接口（这样只要是在写之后获取优先级，就一定是对的），以此尽可能的避免竞争。

另外，不太方便用锁去规避这一个竞争。因为如果这样做，就要在锁被释放之前就调用 `schedule()` 切换线程，之后有可能其他线程也会试图去持有这把锁，造成互相的阻塞甚至死锁。

## Rationale

> C1: Have you considered other design possibilities? You can talk about anything in your solution that you once thought about doing them another way. And for what reasons that you made your choice?

暂时还没考虑别的设计（或者说有一些别的设计，做一半发现不行，于是才改成现在的设计）