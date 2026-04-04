#![no_std]
#![no_main]

use uefi::prelude::*;
use core::arch::asm;
use core::hint::spin_loop;

const PHI: f64 = 1.618033988749895;

extern "C" fn resonance_task(_context: *mut core::ffi::c_void) {
    let mut expansion: f64 = 1.0;
    let mut contraction: f64 = 1.0;

    loop {
        expansion *= PHI;
        contraction /= PHI;

        let _stasis = expansion * contraction;

        unsafe {
            asm!("pause");
        }

        if expansion > 1e15 {
            expansion = 1.0;
            contraction = 1.0;
        }
        
        core::hint::black_box(_stasis);
    }
}

#[entry]
fn main(_image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bt = system_table.boot_services();

    if let Ok(mp_proto) = bt.locate_protocol::<uefi::proto::pi::mp::MpServices>() {
        let mp_services = unsafe { &mut *mp_proto.get() };

        let _ = mp_services.startup_all_aps(
            true,
            resonance_task,
            core::ptr::null_mut(),
            None,
            None,
        );
    }

    resonance_task(core::ptr::null_mut());

    Status::SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        spin_loop();
    }
}
