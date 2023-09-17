#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_string("Hello world");
    write!(vga_buffer::WRITER.lock(), ", the numbers are {} and {}", 42, 1.0/3.0).unwrap();
    write!(vga_buffer::WRITER.lock(), "\nOn a new line:\n:)").unwrap();

    loop {}
}
