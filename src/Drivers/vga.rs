use lazy_static::lazy_static;
use spin::Mutex;

use crate::IO::*;

const VGA_VIDEO_MEMORY: *mut u8 = 0xb8000 as *mut u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum VGAColorCodes{
    Black,
    Blue,
    Green, 
    Cyan,
    Red,
    Magenta,
    Brown, 
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightMagenta,
    Yellow,
    White
}

impl From<u8> for VGAColorCodes{
    fn from(value: u8) -> Self {
        match value{
            0 => Self::Black,
            1 => Self::Blue,
            2 => Self::Green,
            3 => Self::Cyan,
            4 => Self::Red,
            5 => Self::Magenta,
            6 => Self::Brown,
            7 => Self::LightGray,
            8 => Self::DarkGray,
            9 => Self::LightBlue,
            10 => Self::LightGreen,
            11 => Self::LightCyan,
            12 => Self::LightRed,
            13 => Self::LightMagenta,
            14 => Self::Yellow,
            15 => Self::White,
            _ => unimplemented!()
        }
    }
}

pub fn make_color(bg: VGAColorCodes, fg: VGAColorCodes) -> u8{
    (bg as u8) << 4 | fg as u8
}

pub fn extract_fg(color: u8) -> VGAColorCodes{
    let mask = 0b0000_1111;
    VGAColorCodes::from(color & mask)
}

pub fn extract_bg(color: u8) -> VGAColorCodes{
    let mask = 0b1111_0000;
    VGAColorCodes::from(color & mask)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Writer{
    //pub(crate) buffer: *mut u8,
    pub(crate) color: u8,
    pub(crate) fg_color: VGAColorCodes,
    pub(crate) bg_color: VGAColorCodes,
    pub(crate) pos: usize,
}

impl Writer{
    pub fn new(color: u8) -> Self{
        Self {/*buffer: 0xb8000 as *mut u8,*/color: color, fg_color: extract_fg(color), bg_color: extract_bg(color), pos: 0}
    }

    pub fn update_cursor(&mut self) {
        outb(0x3d4, 0x0f);
        outb(0x3d5, (self.pos & 0xff) as u8);
        outb(0x3d4, 0x0e);
        outb(0x3d5, ((self.pos >> 8) & 0xff) as u8);
    }

    fn new_line(&mut self){
        for i in 0..80{

        }
    }

    pub fn putc(&mut self, c: char){
        
        unsafe{
            match c{
                '\n' => {
                    for _ in 0..80{
                        *VGA_VIDEO_MEMORY.offset((self.pos * 2 + 1) as isize) = self.color;
                        self.pos += 1;
                    }
                    self.pos -= self.pos % 80;
                }

                _ => {
                    *VGA_VIDEO_MEMORY.offset((self.pos * 2) as isize) = c as u8;
                    *VGA_VIDEO_MEMORY.offset((self.pos * 2 + 1) as isize) = self.color;
                    self.pos += 1;
                    *VGA_VIDEO_MEMORY.offset((self.pos * 2 + 1) as isize) = self.color;
                }
            }

        }; 
        self.update_cursor();
    }

    pub fn puts(&mut self, s: &str){
        for c in s.chars(){
            self.putc(c);
        }
    }
}

lazy_static!{
    pub(crate) static ref G_WRITER: Mutex<Writer> = {
        Mutex::new(unsafe{Writer::new(make_color(VGAColorCodes::Black, VGAColorCodes::White))})
    };
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result{
        G_WRITER.lock().puts(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let mut writer = Writer::new(make_color(VGAColorCodes::Black, VGAColorCodes::White));
    use core::fmt::Write;
    writer.write_fmt(args).unwrap();
}