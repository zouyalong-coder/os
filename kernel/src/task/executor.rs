use core::{
    panic,
    task::{Context, Waker},
};

use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;

use super::{Task, TaskId};

struct TaskWaker {
    task_id: TaskId,
    /// 与 Executor 共享的队列。
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        // from 函数负责为我们的TaskWaker类型构造一个RawWakerVTable和一个RawWaker实例
        Waker::from(Arc::new(Self {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        // wake 只是简单将它放回 ready 队列即可。
        self.task_queue.push(self.task_id).expect("queue is full")
    }
}

// 实现了 Wake trait，所以 TaskWaker 可以被注册到 Waker 中。
impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task()
    }

    // 节省一次 clone。
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task()
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    /// 用于存放 task id 的队列。由于队列会在 多个 Waker 和 Executor 之间共享，所以这里使用 Arc。ArrayQueue 是无锁、安全的队列，所以不需要 Mutex。
    /// 使用固定大小的队列，避免了内存分配。
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            // 这里依然是忙等的方法
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        if self.task_queue.is_empty() {
            // 这中间可能有键盘中断进入，这会导致该中断的响应要下一个键盘输入才被处理
            x86_64::instructions::hlt();
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            // 由于taskId 是唯一的，这里的 panic 不应该发生。
            panic!("task with same ID already in tasks");
        }
        // 这里会使 task 尽快开始执行。
        self.task_queue.push(task_id).expect("queue full");
    }

    fn run_ready_tasks(&mut self) {
        // 避免借用检查器报错。
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;
        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(t) => t,
                // task 不存在了。
                None => continue,
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                core::task::Poll::Ready(_) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                // 啥都不做，不需要放回去。
                core::task::Poll::Pending => {}
            }
        }
    }
}
