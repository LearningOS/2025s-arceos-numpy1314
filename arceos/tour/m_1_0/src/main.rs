#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
extern crate axstd as std;
extern crate alloc;

#[macro_use]
extern crate axlog;

mod task;
mod syscall;
mod loader;

use axstd::io;
use axhal::paging::MappingFlags;
use axhal::arch::UspaceContext;
// use axhal::mem::VirtAddr;
use axsync::Mutex;
use alloc::sync::Arc;
use axmm::AddrSpace;
use loader::load_user_app;

use axhal::mem::{VirtAddr, virt_to_phys};
use axhal::trap::{register_trap_handler, PAGE_FAULT};

const USER_STACK_SIZE: usize = 0x10000;
const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB
const APP_ENTRY: usize = 0x1000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // A new address space for user app.
    let mut uspace = axmm::new_user_aspace().unwrap();

    // Load user app binary file into address space.
    if let Err(e) = load_user_app("/sbin/origin", &mut uspace) {
        panic!("Cannot load app! {:?}", e);
    }

    // Init user stack.
    // let ustack_top = init_user_stack(&mut uspace, true).unwrap();
    let ustack_top = init_user_stack(&mut uspace, false).unwrap();
    ax_println!("New user address space: {:#x?}", uspace);

    // Let's kick off the user process.
    let user_task = task::spawn_user_task(
        Arc::new(Mutex::new(uspace)),
        UspaceContext::new(APP_ENTRY.into(), ustack_top),
    );

    // Wait for user process to exit ...
    let exit_code = user_task.join();
    ax_println!("monolithic kernel exit [{:?}] normally!", exit_code);
}
use axtask::TaskExtRef;
#[register_trap_handler(PAGE_FAULT)]
fn handle_page_fault(va: VirtAddr, flg: MappingFlags, populating: bool) -> bool {
    let pa = virt_to_phys(va);
    axtask::current().task_ext().aspace.lock().handle_page_fault(va, flg)
}

fn init_user_stack(uspace: &mut AddrSpace, populating: bool) -> io::Result<VirtAddr> {
    let ustack_top = uspace.end();
    let ustack_vaddr = ustack_top - crate::USER_STACK_SIZE;
    // ax_println!(
    //     "Mapping user stack: {:#x?} -> {:#x?}",
    //     ustack_vaddr, ustack_top
    // );
    // uspace.map_alloc(
    //     ustack_vaddr,
    //     crate::USER_STACK_SIZE,
    //     MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
    //     populating,
    // ).unwrap();
    // Ok(ustack_top)

    ax_println!(
        "Mapping user stack: {:#x?} -> {:#x?}",
        ustack_vaddr,
        ustack_top
    );
    uspace
        .map_alloc(
            ustack_vaddr,
            crate::USER_STACK_SIZE,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
            populating,
        )
        .unwrap();
    Ok(ustack_top)
}