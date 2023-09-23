use lazy_static::lazy_static;
use x86_64::structures::idt::{
    DivergingHandlerFuncWithErrCode,
    Entry,
    HandlerFunc,
    HandlerFuncType,
    InterruptDescriptorTable,
    InterruptStackFrame,
};

use crate::gdt;
use crate::println;

pub struct IDTDefinition {
    pub breakpoint: IDTEntryDefinition<HandlerFunc>,
    pub double_fault: IDTEntryDefinition<DivergingHandlerFuncWithErrCode>,
}

pub struct IDTEntryDefinition<F> {
    pub handler_fn: F,
    pub stack_index: Option<gdt::ISTIndex>,
}

impl IDTDefinition {
    pub fn init(&self) -> InterruptDescriptorTable {
        let mut idt = InterruptDescriptorTable::new();
        self.init_idt_handler(&mut idt.breakpoint, &self.breakpoint);
        self.init_idt_handler(&mut idt.double_fault, &self.double_fault);
        idt
    }

    fn init_idt_handler<F: HandlerFuncType + Copy>(
        &self,
        entry: &mut Entry<F>,
        entry_def: &IDTEntryDefinition<F>,
    ) {
        let opts = entry.set_handler_fn(entry_def.handler_fn);
        match &entry_def.stack_index {
            None => (),
            Some(index) => index.set_stack_index(opts),
        }
    }
}

impl Default for IDTDefinition {
    fn default() -> Self {
        IDTDefinition {
            breakpoint: IDTEntryDefinition {
                handler_fn: breakpoint_handler,
                stack_index: None,
            },
            double_fault: IDTEntryDefinition {
                handler_fn: double_fault_handler,
                stack_index: Some(gdt::ISTIndex::DoubleFault),
            },
        }
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = IDTDefinition::default().init();
}

pub fn init() {
    IDT.load();
}

/***** Breakpoint *****/

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

/***** Double fault *****/

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
