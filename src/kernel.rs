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

#[path = "Drivers/Writers/VGA.rs"]
mod VGA;
#[path = "IO/IO.rs"]
mod IO; 
#[path = "Drivers/Setup/IDT.rs"]
mod IDT;
#[path = "Drivers/Setup/GDT.rs"]
mod GDT;
#[path = "Drivers/Setup/interrupts.rs"]
mod interrupts;
#[path = "Memory/Paging/paging.rs"]
mod paging;

mod serial;

//---------------------CRATE USES--------------------------------------

#[allow(dead_code)]
#[allow(unused_imports)]
use crate::VGA::{Colors, cls, CreateColor, SetCursorPos, SetCursorPosFromWH, SetVGAColor, current_color, default_color};
use crate::IDT::init_idt;
use crate::GDT::GDT_init;
use crate::interrupts::PICS;

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
    
    SetCursorPos(0);
    cls(CreateColor(Colors::White, Colors::Black));
    
    SetVGAColor(CreateColor(Colors::Brown, Colors::DarkGray));
    println!("different colored text");
    SetVGAColor(default_color);

    init_idt();

    GDT_init();

    unsafe{PICS.lock().initialize();}

    x86_64::instructions::interrupts::enable();
    
    //int3(); //breakpoint
    
    /*let dead_mem: *mut u64 = 0xdeadbeef as *mut u64;
    unsafe{*dead_mem = 9;} //0x2093b1*/

    
    let phys_offset = VirtAddr::new(bootinfo.physical_memory_offset);

    let mapper = unsafe{paging::Offset_page_table_init(phys_offset)};

    let adresses = [
        0xb8000, 0x201008, 0x0100_0020_1a10, bootinfo.physical_memory_offset,
    ];

    for &addr in &adresses {
        let virt_addr = VirtAddr::new(addr);
        let phys_addr = mapper.translate_addr(virt_addr);
        println!("Virtual adress:{:?} to physical adress: {:?}", virt_addr, phys_addr);
    }
    
    #[cfg(test)]
    test_main();

    println!("I survived?");

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
    SetVGAColor(CreateColor(Colors::Red, Colors::Black));
    println!("");
    println!("RenseOS kernel panicked for some reason, here is the panic info:\n{}", _info);
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