#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![feature(iter_intersperse)]

mod sbi;

extern crate alloc;
extern crate allocator;

use alloc::string::String;
use timestamp::DateTime;
use core::{
    fmt::{self, Write},
    panic::PanicInfo, ptr::read_volatile,
};
use fdt::Fdt;
use sbi::{console_putchar, shutdown};

/// RISCV boot: OpenSBI -> OS, a0: hart_id, a1: device_tree

/// RISCV 有三个态 M(SBI) S(OS) U(用户程序)

/// Boot(启动)
/// 输出Hello World!
/// 初始化内存

/// 堆栈大小
const STACK_SIZE: usize = 0x8_0000; // 1k * 16 * 8, 128k

/// 默认堆栈
#[link_section = ".bss.stack"]
static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

/// 汇编入口函数
///
/// 分配栈 初始化页表信息 并调到rust入口函数
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        // 1. 设置栈信息
        "
            la      sp, {boot_stack}
            li      t0, {stack_size}
            add     sp, sp, t0              // set boot stack
            call    main
        ",
        stack_size = const STACK_SIZE,
        boot_stack = sym STACK,
        options(noreturn),
    )
}

fn puts(display_str: &str) {
    for c in display_str.as_bytes() {
        console_putchar(*c);
    }
}

#[inline]
fn clear_bss() {
    unsafe {
        extern "C" {
            fn _sbss();
            fn _ebss();
        }

        let start = _sbss as usize;
        let end = _ebss as usize;
        core::slice::from_raw_parts_mut(start as *mut u8, end - start).fill(0);
    }
}

fn get_program_size() -> usize {
    extern "C" {
        fn _skernel();
        fn _ekernel();
    }
    _ekernel as usize - _skernel as usize
}

#[no_mangle]
fn main(hart_id: usize, device_tree: usize) -> ! {
    clear_bss();

    allocator::init();

    puts(include_str!("banner.txt"));
    println!("boot hart: {}", hart_id);
    println!("program size: {} KB", get_program_size() / 1024);
    println!("device_tree addr: {:#x}", device_tree); // 0x 十六进制， 0o 八进制， 0b 二进制

    let fdt = unsafe {
        Fdt::from_ptr(device_tree as *const u8).expect("This is a not a valid device tree")
    };

    println!(
        "Platform: {}  {} CPU(s)",
        fdt.root().model(),
        fdt.cpus().count()
    );
    fdt.memory().regions().for_each(|x| {
        println!(
            "Memory region {:#x} - {:#x}",
            x.starting_address as usize,
            x.starting_address as usize + x.size.unwrap()
        );
    });
    fdt.all_nodes().for_each(|child| {
        if let Some(compatible) = child.compatible() {
            println!(
                "{}  {}",
                child.name,
                compatible.all().intersperse(" ").collect::<String>()
            );
            if let Some(_) = compatible.all().find(|x| *x == "google,goldfish-rtc") {
                let base_addr = child.reg().unwrap().next().unwrap().starting_address as usize;
                let timestamp = unsafe {
                    let low: u32 = read_volatile((base_addr + 0x0) as *const u32);
                    let high: u32 = read_volatile((base_addr + 0x4) as *const u32);
                    ((high as u64) << 32) | (low as u64) 
                } / 1_000_000_000u64;
                let dt = DateTime::new(timestamp as usize);
                println!("dt: {:#?}", dt);
            }
        }
    });

    shutdown()
}

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        puts(s);
        Ok(())
    }
}

// 程序遇到错误
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // puts("");
    // Logger.write_fmt(*info.message().unwrap());
    println!("An error occurred: {}", info.message().unwrap());
    shutdown()
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

#[inline]
pub fn print(args: fmt::Arguments) {
    Logger
        .write_fmt(args)
        .expect("can't write string in logging module.");
}
