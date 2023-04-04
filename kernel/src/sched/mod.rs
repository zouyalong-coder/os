pub mod scheduler;

pub enum State {
    /// The task is runnable. Running or ready to run.
    Runnable,
    /// 任务被阻塞，等待某个事件发生
    Interruptible,
    /// 任务被阻塞，等待某个事件发生，但是在等待期间不会被中断
    Uninterruptible,
}

pub enum ExitState {
    /// 任务结束，但是还没有被回收，等待父进程执行 wait4 系统调用
    Zombie,
    /// 任务结束.
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(pub u32);

/// 同 linux 的 task_struct 结构体。
pub struct TaskStruct {
    state: State,
    exit_state: ExitState,
    pid: Pid,

    /// 任务的内核栈地址。
    kernel_stack_addr: u64,
}
