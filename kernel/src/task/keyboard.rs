use core::{
    pin::Pin,
    task::{Context, Poll},
};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Stream, StreamExt};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

use crate::{print, println};

/// 用于存储键盘输入的队列.
/// 使用 OnceCell 来保证只初始化一次，不用 lazy_static! 宏的原因：保证初始化时执行，如果在中断时调用，则会在中断处理程序中发生 heap 分配，这是不安全的，由于分配会上锁，可能导致死锁。
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
/// 基于原子操作，可以保证并发。
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scan_code(scan_code: u8) {
    // try_get 获取队列，如果队列未初始化，则返回 Err。而不会在这里初始化，因为初始化需要分配内存，而分配内存是不安全的。
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scan_code) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            // 有新数据，唤醒等待的任务。
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScanCodeStream {
    /// 避免 module 外部创建实例
    _private: (),
}

impl ScanCodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("Scancode queue already initialized");
        Self { _private: () }
    }
}

impl Stream for ScanCodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }
        WAKER.register(&cx.waker());
        // 二次查询。这里可能在 WAKER 注册时，队列中已经有数据了，所以需要再次查询。
        match queue.pop() {
            Ok(scan_code) => {
                WAKER.take(); // 从 WAKER 中移除，因为已经有数据了。
                Poll::Ready(Some(scan_code))
            }
            // 二次检查仍然没有，返回 Pending。Waker 会被唤醒。
            Err(_) => Poll::Pending,
        }
        // match queue.pop() {
        //     Ok(scancode) => Poll::Ready(Some(scancode)),
        //     Err(crossbeam_queue::PopError) => {
        //         // 队列为空，注册中断，等待下一次中断
        //         cx.waker().wake_by_ref();
        //         Poll::Pending
        //     }
        // }
    }
}

pub async fn print_keypress() {
    let mut scan_codes = ScanCodeStream::new();
    let mut keyboard = Keyboard::new(
        layouts::Us104Key, // 默认美式键盘布局
        ScancodeSet1,
        HandleControl::Ignore, // 使 Ctrl 键不会影响输入
    );

    while let Some(scan_code) = scan_codes.next().await {
        // KeyEvent 包括了触发本次中断的按键信息，以及子动作是按下还是释放。
        if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
            // process_keyevent 的作用是将按键转换为人类可读的字符，比如shift 同时按下时将按键 a 转换为字符 'A'。
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
