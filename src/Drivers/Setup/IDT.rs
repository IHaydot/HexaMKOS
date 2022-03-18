use x86_64::structures::idt::{InterruptStackFrame ,InterruptDescriptorTable, PageFaultErrorCode};
use lazy_static::lazy_static;
use crate::{println, print, hlt};
use crate::VGA::{Colors ,SetVGAColor, CreateColor, default_color, cls, current_color, CURSOR_POS, SetCursorPos};
use crate::GDT::DOUBLE_FAULT_IST_INDEX;
use crate::interrupts::{InterruptIndex, PICS};
use x86_64::instructions::port::Port;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyEvent};
use spin;
use x86_64::registers::control::Cr2;

lazy_static!{
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.divide_error.set_handler_fn(divide_handler);
        unsafe{idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);}
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(ps2_keyboard_handler);
        idt
    };
}

pub fn init_idt(){
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame){
    cls(default_color);
    SetVGAColor(CreateColor(Colors::Yellow, Colors::Black));
    println!("THE OPERATING SYSTEM HAS HIT AN EXCEPTION: BREAKPOINT. PRESS ENTER TO CONTINUE. STACK FRAME:\n{:#?}", stack_frame);
    SetVGAColor(default_color);
}

extern  "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode){
    cls(CreateColor(Colors::Black, Colors::Red));
    unsafe{SetVGAColor(current_color);}
    println!("THE OPERATING SYSTEM HAS HIT AN EXCEPTION: PAGE FAULT. UNABLE TO CONTINUE.");
    println!("ADRESS ACCESSED: {:#?}", Cr2::read());
    println!("PAGE FAULT ERROR CODE: {:#?}", error_code);
    println!("STACK FRAME: {:#?}", stack_frame);
    hlt();
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> !{
    cls(CreateColor(Colors::Black, Colors::Red));
    unsafe{SetVGAColor(current_color);}
    println!("THE OPERATING SYSTEM HAS HIT A DOUBLE FAULT: NO POSSIBLE WAY OF CONTINUING.");
    println!("DOUBLE FAULT ERROR CODE: {:#?}", error_code);
    println!("STACK FRAME: {:#?}", stack_frame);
    hlt();
}

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame){
    unsafe{
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn ps2_keyboard_handler(stack_frame: InterruptStackFrame){
    let mut read_port = Port::new(0x60);
    let scan_code: u8 = unsafe {read_port.read()};

    lazy_static!{
        static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            spin::Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut Keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = Keyboard.add_byte(scan_code){
        if let Some(key) = Keyboard.process_keyevent(key_event){
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => if scan_code == 0x08
                {unsafe{CURSOR_POS -= 1;
                print!(" ");
                CURSOR_POS -= 1;
                SetCursorPos(CURSOR_POS);}
                }else{ print!("{:?}", key)}
            }
        }
    }


    unsafe{
        unsafe{
            PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
        }
    }
}

extern "x86-interrupt" fn divide_handler(stack_frame: InterruptStackFrame){
    cls(CreateColor(Colors::Black, Colors::Red));
    println!("THE OPERATING SYSTEM HAS HIT A DIVIDE BY ZERO FAULT.");
    println!("STACK FRAME: {:#?}", stack_frame);
    hlt();
}