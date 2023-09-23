#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(toy_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use toy_os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World! {}", ":)");

    toy_os::init();

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

     println!("Looping...");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    toy_os::test_panic_handler(info)
}

#[test_case]
fn smoketest() {
    assert_eq!(1, 1);
}
