#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![feature(iter_intersperse)]

mod frame;
mod logging;
mod sbi;

#[macro_use]
extern crate alloc;
extern crate allocator;

use alloc::string::String;
use log::{debug, error, info, trace, warn};
use timestamp::DateTime;
use core::{
    fmt::{self, Write},
    panic::PanicInfo, ptr::read_volatile,
};
use fdt::Fdt;
use sbi::{console_putchar, shutdown};

use crate::frame::add_frame_area;

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

fn get_kernel_range() -> (usize, usize) {
    extern "C" {
        fn _skernel();
        fn _ekernel();
    }
    (_skernel as usize, _ekernel as usize)
}


#[no_mangle]
fn main(hart_id: usize, device_tree: usize) -> ! {
    clear_bss();

    allocator::init();
    // env: Environment
    logging::init(option_env!("LOG"));

    puts(include_str!("banner.txt"));

    trace!("Hello Trace");
    debug!("Hello Debug");
    info!("Hello Info");
    warn!("Hello Warn");
    error!("Hello Error");

    info!("boot hart: {}", hart_id);
    info!("program size: {} KB", (get_kernel_range().1 - get_kernel_range().0) / 1024);
    info!("program range: {:#x} - {:#x}", get_kernel_range().0, get_kernel_range().1);
    info!("device_tree addr: {:#x}", device_tree); // 0x 十六进制， 0o 八进制， 0b 二进制

    let fdt = unsafe {
        Fdt::from_ptr(device_tree as *const u8).expect("This is a not a valid device tree")
    };

    info!(
        "Platform: {}  {} CPU(s)",
        fdt.root().model(),
        fdt.cpus().count()
    );

    // 1024 1k  0x1000 4k 0x8000000 / 0x1000 = 0x8000 * 4Kb 8 * 4K * 4k = 4*4*8 = 16 * 8 = 128M
    // x86_64 段式内存管理 页式内存管理  页式 4K
    
    fdt.all_nodes().for_each(|child| {
        if let Some(compatible) = child.compatible() {
            info!(
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
                info!("dt: {:#?}", dt);
            }
        }
    });

    let mut mem_start = 0;
    let mut mem_size = 0;

    fdt.memory().regions().for_each(|x| {
        info!(
            "Memory region {:#x} - {:#x}",
            x.starting_address as usize,
            x.starting_address as usize + x.size.unwrap()
        );
        mem_start = get_kernel_range().1;
        mem_size = x.size.unwrap() - (get_kernel_range().1 - 0x8000_0000);
    });
    
    add_frame_area(mem_start, mem_size);

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
    error!("An error occurred: {}", info.message().unwrap());
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
