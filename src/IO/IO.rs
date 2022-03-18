use core::arch::*;

pub fn outb(port: u16, data: u8){
    unsafe{
        asm!("out dx, al",
        in("dx") port,
        in("al") data
        );
    }
}
pub fn inb(port: u16) -> u8{
    let mut returnValue: u8 = 0;
    unsafe{
        asm!("in dx, al",
        in("dx") port,
        in("al") returnValue
        );
        return returnValue;
    }
}