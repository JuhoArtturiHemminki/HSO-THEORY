#![no_std]
#![no_main]

use uefi::prelude::*;

const PHI: f64 = 1.618033988749895;
const H_C: f64 = 5.0832;
const K1: f64 = 0.042;
const K0: f64 = 0.011;
const K_L1: f64 = 0.08832;
const K_L2: f64 = 0.02211;

pub struct HSISearcher {
    pub current_k: f64,
    pub locked: bool,
}

impl HSISearcher {
    pub fn new() -> Self {
        Self {
            current_k: K1,
            locked: false,
        }
    }

    pub fn tune(&mut self, flux: f64) {
        if flux > 0.00001 {
            self.current_k -= (H_C / PHI) * 0.0000001;
            self.locked = false;
        } else {
            self.locked = true;
        }
    }
}

pub struct CacheModulator {
    pub l1_phase: f64,
    pub l2_phase: f64,
    pub sync_locked: bool,
}

impl CacheModulator {
    pub fn new() -> Self {
        Self {
            l1_phase: K_L1,
            l2_phase: K_L2,
            sync_locked: false,
        }
    }

    pub fn modulate(&mut self, resonance: f64) {
        if (resonance % PHI) > 0.0000314 {
            self.l1_phase = (K_L1 * PHI) / H_C;
            self.l2_phase = (K_L2 * PHI) / (H_C * 2.0);
            self.sync_locked = false;
        } else {
            self.l1_phase = K_L1;
            self.l2_phase = K_L2;
            self.sync_locked = true;
        }
        unsafe {
            let l1_val = (self.l1_phase * 1000000.0) as u32;
            let l2_val = (self.l2_phase * 1000000.0) as u32;
            core::arch::asm!("wrmsr", in("ecx") 0x1A0, in("eax") l1_val, in("edx") l2_val);
        }
    }
}

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let mut searcher = HSISearcher::new();
    let mut cache = CacheModulator::new();
    let f_base = 100.0 / PHI;

    unsafe {
        core::arch::asm!("cli");
        let msr_perf: u32 = 0x199;
        core::arch::asm!("wrmsr", in("ecx") msr_perf, in("eax") 0, in("edx") 0);
    }

    loop {
        let thermal_raw: u64;
        unsafe {
            let msr_temp: u32 = 0x19C;
            core::arch::asm!("rdmsr", in("ecx") msr_temp, out("eax") thermal_raw, out("edx") _);
        }

        searcher.tune((thermal_raw as f64) % 1.0);
        let resonance = (f_base / PHI) * H_C * (1.0 + searcher.current_k);
        cache.modulate(resonance);

        unsafe {
            let v_tune = (resonance * 0.001) as u32;
            core::arch::asm!("out dx, eax", in("dx") 0xCF8, in("eax") v_tune);
            
            let vga_sync = ((f_base * H_C) / (PHI * PHI)) as u32;
            core::arch::asm!("out dx, al", in("dx") 0x3C8, in("al") vga_sync as u8);
        }

        if searcher.locked && cache.sync_locked {
            unsafe {
                core::arch::asm!("wbinvd");
                break;
            }
        }
    }

    Status::SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
