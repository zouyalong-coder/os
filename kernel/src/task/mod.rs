pub mod keyboard;
pub mod simple_executor;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::boxed::Box;

pub struct Task {
    ///
    /// Pin: 不被 move，不允许获取 &mut 引用。
    /// Box: 分配在堆上
    /// dyn Future: 动态分发，且为 Future。
    /// Pin<Box>: 保证自引用的安全性。
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    // 'static: Task 可能存在任意时间（直到被 poll 并且完成）
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            // 注意：这里实际上发生了一次move，Box::new 会将 future move 到堆上。由于future 在被 poll 前是没有自引用的，所以是可以 move 的。
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        // 对 Future 进行 poll，Future 由编译器生成。
        self.future.as_mut().poll(context)
    }
}
