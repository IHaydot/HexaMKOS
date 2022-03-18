use bootloader::BootInfo;
use x86_64::structures::paging::{page_table::{PageTableEntry, FrameError, PageTableFlags as flags}, PageTable, OffsetPageTable, Page, PhysFrame, Mapper, Size4KiB, FrameAllocator};
use x86_64::{VirtAddr, PhysAddr};
use x86_64::registers::control::Cr3;
use crate::{print, println, panic};

unsafe fn active_level_4_table(phys_offset: VirtAddr) -> &'static mut PageTable{
    let (level_4_table_frame, _) = Cr3::read();

    let phys_addr = level_4_table_frame.start_address();
    let virt_addr = phys_offset + phys_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();

    &mut *page_table_ptr
}

///Currently only gives the count of PTL4 and PTL3
fn paging_info(bootinfo: &'static BootInfo){

    let phys_mem_offset = VirtAddr::new(bootinfo.physical_memory_offset);
    let level_4_table = unsafe {active_level_4_table(phys_mem_offset)};
    let mut total_page_tables4 = 0;
    let mut total_page_tables3 = 0;
    for (i, entry) in level_4_table.iter().enumerate() {
        total_page_tables4 += 1;
        if !entry.is_unused(){
            //println!("PTL4 entry {} at: {:?}", i, entry);

            let phys_addr = entry.frame().unwrap().start_address();
            let virt_addr = phys_addr.as_u64() + bootinfo.physical_memory_offset;
            let ptr = VirtAddr::new(virt_addr).as_mut_ptr();
            let PTL3_table: &PageTable = unsafe{&*ptr};

            for (i3, entry3) in PTL3_table.iter().enumerate(){
                total_page_tables3 += 1;
                if !entry3.is_unused(){
                    //println!("In PTL4 number {}, found PTL3 number {} with start adress {:?}", i, i3, entry3);
                }
            }

        }
    }
    println!("Total PTL4 found: {}", total_page_tables4);
    println!("Total PTL3 found: {}", total_page_tables3);
}

pub unsafe fn Offset_page_table_init(phys_offset: VirtAddr) -> OffsetPageTable<'static>{
    let PTL4 = active_level_4_table(phys_offset);
    OffsetPageTable::new(PTL4, phys_offset)
}

pub fn CreatePage(page: Page, frame: &mut impl FrameAllocator<Size4KiB>, mapper: &mut OffsetPageTable){
    let inner_frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = flags::PRESENT | flags::WRITABLE;
    let map_result = unsafe{
        mapper.map_to(page, inner_frame, flags, frame)
    };
    map_result.expect("Failed to map adress 0xb8000").flush();
}

pub struct EmptyFrameAlloc;
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAlloc{
    fn allocate_frame(&mut self) -> Option<PhysFrame>{
        None
    }
}