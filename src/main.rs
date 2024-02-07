#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(asm_const)]

mod sbi;

use core::{fmt::{self, Write}, panic::PanicInfo};
use sbi::{console_putchar, shutdown};

/// RISCV 有三个态 M(SBI) S(OS) U(用户程序)

/// 堆栈大小
const STACK_SIZE: usize = 0x80000;

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

#[no_mangle]
fn main() -> ! {
    clear_bss();

    puts(include_str!("banner.txt"));
    println!("Hello World!");

    let t = Option::<usize>::default();
    t.expect("I confirm this can be unwraped");

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