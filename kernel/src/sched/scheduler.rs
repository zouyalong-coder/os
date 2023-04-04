use super::TaskStruct;

/// 同 struct sched_class(include/linux/sched.h)
pub trait Scheduler {
    fn enqueue_task(&mut self, task: TaskStruct);
    fn dequeue_task(&mut self) -> Option<TaskStruct>;
}
