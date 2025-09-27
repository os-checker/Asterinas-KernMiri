#![feature(start)]
#![feature(generic_const_exprs)]

#![no_std]
#![cfg_attr(not(miri), no_main)]

//extern crate ostd;

#[cfg(miri)]
#[start]
fn miri_start(_argc: isize, _argv: *const *const u8) -> isize {
    ostd::arch::boot::miri_boot();

    return 0;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    extern "Rust" {
        pub fn __ostd_panic_handler(info: &core::panic::PanicInfo) -> !;
    }
    unsafe { __ostd_panic_handler(info); }
}
