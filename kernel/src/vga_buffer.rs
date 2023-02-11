//! 参考 https://os.phil-opp.com/zh-CN/vga-text-mode/

use core::fmt;

use volatile::Volatile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // 以 u8 的形式存储，而不是默认的 i32
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// VGA 文本模式的颜色代码
// 定义了字符的显示方式，前四个比特定义了前景色，中间三个比特定义了背景色，最后一个比特则定义了该字符是否应该闪烁
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // 确保 ColorCode 和 u8 有完全相同的内存布局
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        // 0~4 位是前景色(4位是加亮位)，5~7 位是背景色, 8 位是闪烁
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // 保证结构体的字段按照声明的顺序布局
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// VGA 文本模式的缓冲区，默认大小为 25 行 80 列
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)] // 保证内存结构与内部 chars 属性一致，即没有额外的内存填充
struct Buffer {
    // 使用 Volatile 标记数据为易变的，防止编译器优化掉没有用到的写操作
    // 由于我们不会再 Rust 中读取屏幕缓冲区的内容，所以可能被编译器优化掉
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// 将字符写入屏幕的最后一行，并在一行写满或接收到换行符 \n 的时候，将所有的字符向上位移一行
pub struct Writer {
    column_position: usize,      // 跟踪光标在最后一行的位置
    color_code: ColorCode,       // 当前字符的前景和背景色
    buffer: &'static mut Buffer, // 这个借用应该在整个程序的运行期间有效
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // 换行
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可打印 ASCII 字符或空格
                // VGA 字符缓冲区只支持 ASCII 码字节和代码页 437（Code page 437）定义的字节
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 不可打印 ASCII 字符
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static::lazy_static! {
    // 0xb8000 是 VGA 文本缓冲区的起始地址，使用内存映射的方式。
    // 此时没有 Mutex，使用 spin Mutex 代替。spin Mutex 与 std Mutex 的区别在于，spin Mutex 不会阻塞线程，而是在等待锁的时候一直循环检查锁是否可用
    pub static ref WRITER: spin::Mutex<Writer> = spin::Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// 不在文档中生成这个函数。
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export] // 使得 print! 和 println! 宏可以在其他模块中使用
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga_buffer::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_println_output() {
        let s = "Some test string that fits on a single line";
        println!("{}", s);
        for (i, c) in s.chars().enumerate() {
            let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    }
}
