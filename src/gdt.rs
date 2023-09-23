use lazy_static::lazy_static;

use x86_64::VirtAddr;
use x86_64::structures::idt;
use x86_64::structures::tss::TaskStateSegment;

#[derive(Clone, Copy)]
pub enum ISTIndex {
    DoubleFault = 0,
}

impl ISTIndex {
    pub fn set_stack_index(&self, opts: &mut idt::EntryOptions) {
        unsafe {
            opts.set_stack_index(*self as u16);
        }
    }
}

fn new_interrupt_stack() -> VirtAddr {
    const STACK_SIZE: usize = 4096 * 5;
    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

    let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
    let stack_end = stack_start + STACK_SIZE;
    stack_end
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        let indexes = [
            ISTIndex::DoubleFault,
        ];
        for index in indexes {
            tss.interrupt_stack_table[index as usize] = new_interrupt_stack();
        }
        tss
    };
}

use x86_64::structures::gdt::{GlobalDescriptorTable, SegmentSelector};

struct GlobalDescriptorTableWithSelectors {
    table: GlobalDescriptorTable,
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: GlobalDescriptorTableWithSelectors = {
        use x86_64::structures::gdt::Descriptor;

        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        GlobalDescriptorTableWithSelectors {
            table: gdt,
            code_selector,
            tss_selector,
        }
    };
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.table.load();
    unsafe {
        CS::set_reg(GDT.code_selector);
        load_tss(GDT.tss_selector);
    }
}
