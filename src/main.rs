#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;

mod drivers;
mod macros;
mod gdt;
mod interrupts;
pub mod cli;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

fn init() {
    interrupts::init_idt();
    gdt::init();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    println!("#########################");
    println!("<         WhyOS         >");
    println!("#                       #");
    println!("#     v0.0.1  alpha     #");
    println!("#########################\n");

    crate::cli::CLI.lock().activate();

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}