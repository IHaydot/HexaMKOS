//----------------------PREPROCESSOR OPTIONS-------------------------
#![allow(non_snake_case)]
#![feature(type_ascription)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![allow(warnings)]

//---------------------EXTERN USES-------------------------------------
use core::{panic::PanicInfo};
use x86_64::{VirtAddr, PhysAddr};
use x86_64::instructions::port::Port;
use x86_64::instructions::interrupts::{int3};
use x86_64::registers::control::Cr3;
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::{PageTable, Translate};

//---------------------MODS--------------------------------------------

mod serial;

#[path = "Drivers/vga.rs"]
mod vga;
use vga::*;

#[path = "IO/IO.rs"]
mod IO;

//---------------------CRATE USES--------------------------------------


//--------------------VARIABLES--------------------------------------------


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode{
    Success = 0x10,
    Fail = 0x11,
}

//---------------------CODE--------------------------------------------

entry_point!(kernel_start);

fn kernel_start(bootinfo: &'static BootInfo) -> ! {
    
    println!("Hello {} world", 2);
    
    hlt();
}

//---------------------EXTRA FUNCTIONS-----------------------

fn test_stack_overflow(){
    test_stack_overflow();
}

pub fn ExitQemu(exit_code: QemuExitCode){
    unsafe{
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    ExitQemu(QemuExitCode::Success);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> !{
    //cls(CreateColor(Colors::Black, Colors::Red));
    //SetVGAColor(CreateColor(Colors::Red, Colors::Black));
    //println!("");
    //println!("RenseOS kernel panicked for some reason, here is the panic info:\n{}", _info);
    hlt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    ExitQemu(QemuExitCode::Fail);
    hlt();
}

pub trait Testable {
    fn run(&self) -> ();
}
impl<T> Testable for T
where
T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn hlt() -> !{
    loop{
        x86_64::instructions::hlt();
    }
}

//----------------------------------TEST CASES----------------------------

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
#[test_case]
fn check_exception_breakpoints(){
    int3();
}