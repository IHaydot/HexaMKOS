use crate::IO::*;
use core::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colors{
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


pub static mut current_color: u8 = CreateColor(Colors::White, Colors::Black);
pub const default_color: u8 = CreateColor(Colors::White, Colors::Black);
#[allow(dead_code)]
static TEST_STRING: &[u8] = b"Hello World!";

pub fn RprintFNT() {
    let VIDEO_MEMORY_START = 0xb8000 as *mut u8;
    for (i, &byte) in TEST_STRING.iter().enumerate() {
        unsafe {
            *VIDEO_MEMORY_START.offset(i as isize * 2) = byte;
            *VIDEO_MEMORY_START.offset(i as isize * 2 + 1) = 0xb;
            SetCursorPosFromWH(9, 20);
        }
    }
}

pub const fn CreateColor(foreground: Colors, background: Colors) -> u8{
    return (background as u8) << 4 | (foreground as u8);

}

pub static mut CURSOR_POS: u16 = 0;

pub fn SetCursorPos(pos: u16) {
    outb(0x3d4, 0x0f);
    outb(0x3d5, (pos & 0xff) as u8);
    outb(0x3d4, 0x0e);
    outb(0x3d5, ((pos >> 8) & 0xff) as u8);
    unsafe {
        CURSOR_POS = pos;
    }
}
pub fn SetCursorPosFromWH(width: u16, height: u16) {
    let pos = height * 80 + width;
    SetCursorPos(pos);
}

pub fn VGA_write(message: &str, color: u8) {
    let VIDEO_MEM_START = 0xb8000 as *mut u8;

    let mut c = color;

    if color == 0{
        unsafe{
            c = current_color;
        }
    }

    for byte in message.bytes() {
        unsafe {
            /*if byte == 10{
                CURSOR_POS += 80;
                CURSOR_POS -= CURSOR_POS % 80;
            }*/
            match byte {
                10 => {
                    //CURSOR_POS += 80;
                    for i in 0 .. 80{
                        *VIDEO_MEM_START.offset(CURSOR_POS as isize * 2 + 1) = c;
                        CURSOR_POS += 1;
                    }
                    CURSOR_POS -= CURSOR_POS % 80;
                    break;    
                }
                _ => {*VIDEO_MEM_START.offset(CURSOR_POS as isize * 2) = byte;
                    *VIDEO_MEM_START.offset(CURSOR_POS as isize * 2 + 1) = c;
                    CURSOR_POS += 1;
                }
            }
        }
    }
    unsafe {
        SetCursorPos(CURSOR_POS);
    }
}

#[allow(dead_code)]
pub fn RenseErr(message: &str){
    VGA_write(message, CreateColor(Colors::Black, Colors::Red));
}

pub struct Writer{

}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result{
        VGA_write(s, 0);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::VGA::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}



#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut writer = Writer{

    };
    use core::fmt::Write;
    writer.write_fmt(args).unwrap();
}

pub fn cls(color: u8){
    for i in 0xb8000 .. 0xb8000 + 4000{
        VGA_write(" ", color);
    }
    SetCursorPos(0);
    unsafe{
        current_color = color;
    }
}

pub fn SetVGAColor(color: u8){
    unsafe{
        current_color = color;
    }
}


