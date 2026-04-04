#![no_std]
#![no_main]
#![feature(abi_efiapi)]

use core::arch::asm;
use core::panic::PanicInfo;

pub const PHI: f64 = 1.618033988749895;
pub const H_C: f64 = 5.0832104;
pub const VGA_XP_SYNC: u8 = 0x1F;

pub unsafe fn resonance_hardening_sequence() -> ! {
    asm!("cli", options(nomem, nostack));

    asm!(
        "out dx, al",
        in("dx") 0x3C8,
        in("al") VGA_XP_SYNC,
        options(nomem, nostack)
    );

    write_msr(0x199, 0x13D4);
    write_msr(0x1A0, 0x0800);
    write_msr(0xCE, 0x80000000);

    asm!("wbinvd", options(nomem, nostack));

    loop {
        asm!("pause", options(nomem, nostack));
    }
}

#[inline(always)]
unsafe fn write_msr(msr: u32, value: u64) {
    let low = (value & 0xFFFFFFFF) as u32;
    let high = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack)
    );
}

#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: u64, _system_table: u64) -> u64 {
    unsafe {
        resonance_hardening_sequence();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { asm!("pause"); }
    }
}
