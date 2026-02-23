#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;
use bootloader::BootInfo;
use x86_64::VirtAddr;
use drivers::vga_buffer;

mod drivers;
mod os_info;
mod macros;
mod gdt;
mod interrupts;
pub mod cli;
pub mod memory;
pub mod allocator;
pub mod task;
mod ramfs;

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use crate::drivers::vga_buffer::Color;

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
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    init();

    println!("          _            ____   _____ ");
    println!("         | |          / __ \\ / ____|");
    println!("__      _| |__  _   _| |  | | (___  ");
    println!("\\ \\ /\\ / / '_ \\| | | | |  | |\\___ \\ ");
    println!(" \\ V  V /| | | | |_| | |__| |____) |");
    println!("  \\_/\\_/ |_| |_|\\__, |\\____/|_____/ ");
    println!("                 __/ |              ");
    println!("                |___/    v{}     \n", os_info::VERSION);

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    vga_buffer::WRITER.lock().set_custom_color_code(vga_buffer::ColorCode::new(Color::Cyan, Color::Black));

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

    println!(
        "memory allocated to OS is about {} bytes or {} KiB.",
        allocator::HEAP_SIZE,
        allocator::HEAP_SIZE / 1024
    );

    vga_buffer::WRITER.lock().set_custom_color_code(vga_buffer::ColorCode::new(Color::Green, Color::Black));

    ramfs::mkdir("/", "/home");
    ramfs::create_file("/","/home/test.txt", b"This is a test file made at startup in RamFS.");

    crate::cli::CLI.lock().activate();

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga_buffer::WRITER.lock().set_custom_color_code(vga_buffer::ColorCode::new(Color::Red, Color::Black));
    println!("{}", info);
    hlt_loop();
}