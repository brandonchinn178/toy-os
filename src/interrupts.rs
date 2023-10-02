use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::{
    DivergingHandlerFuncWithErrCode,
    Entry,
    HandlerFunc,
    HandlerFuncType,
    InterruptDescriptorTable,
    InterruptStackFrame,
};

use crate::gdt;
use crate::{print, println};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 40;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    // 0 - 31: IDT
    DivideError = 0,
    // 32 - 39: PIC 1 (Primary)
    Timer = PIC_1_OFFSET,
    Keyboard,
    // 40 - 47: PIC 2 (Secondary)
    RealTimeClock = PIC_2_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }

    fn from_idt(self, idt: &mut InterruptDescriptorTable) -> &mut Entry<HandlerFunc> {
        &mut idt[self.as_usize()]
    }
}

/***** Interrupt Descriptor Table ****/

pub struct IDTDefinition {
    pub breakpoint: IDTEntryDefinition<HandlerFunc>,
    pub double_fault: IDTEntryDefinition<DivergingHandlerFuncWithErrCode>,
    pub timer: IDTEntryDefinition<HandlerFunc>,
    pub keyboard: IDTEntryDefinition<HandlerFunc>,
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
        self.init_idt_handler(InterruptIndex::Timer.from_idt(&mut idt), &self.timer);
        self.init_idt_handler(InterruptIndex::Keyboard.from_idt(&mut idt), &self.keyboard);
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
            timer: IDTEntryDefinition {
                handler_fn: timer_interrupt_handler,
                stack_index: None,
            },
            keyboard: IDTEntryDefinition {
                handler_fn: keyboard_interrupt_handler,
                stack_index: None,
            },
        }
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = IDTDefinition::default().init();
}

/***** Programmable interrupt controller *****/

pub struct ChainedPicsMutex {
    pics: Mutex<pic8259::ChainedPics>,
}

impl ChainedPicsMutex {
    const fn new() -> ChainedPicsMutex {
        let chained_pics = unsafe {
            pic8259::ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
        };
        ChainedPicsMutex {
            pics: Mutex::new(chained_pics),
        }
    }

    fn init(&self) {
        unsafe { self.pics.lock().initialize() };
        x86_64::instructions::interrupts::enable();
    }

    /**
     * Send an end-of-interrupt signal. InterruptIndex must match the interrupt
     * that was just handled.
     */
    unsafe fn send_eoi(&self, index: InterruptIndex) {
        self.pics.lock().notify_end_of_interrupt(index.as_u8());
    }
}

pub static PICS: ChainedPicsMutex = ChainedPicsMutex::new();

/***** Initialize interrupts *****/

pub fn init() {
    IDT.load();
    PICS.init();
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

/***** Timer *****/

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    print!(".");
    unsafe { PICS.send_eoi(InterruptIndex::Timer) };
}

/***** Keyboard *****/

use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(
            Keyboard::new(
                layouts::Us104Key,
                ScancodeSet1,
                HandleControl::Ignore,
            ),
        );
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    use crate::ports;
    use pc_keyboard::DecodedKey;

    let mut keyboard = KEYBOARD.lock();

    let scancode = unsafe { ports::PS2.get_port().read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe { PICS.send_eoi(InterruptIndex::Keyboard) };
}
