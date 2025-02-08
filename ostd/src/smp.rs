// SPDX-License-Identifier: MPL-2.0

//! Symmetric Multi-Processing (SMP) support.
//!
//! This module provides a way to execute code on other processors via inter-
//! processor interrupts.

use core::sync::atomic::Ordering;

use alloc::collections::VecDeque;

use spin::Once;

use crate::{
    arch::kern_miri_init_ap, boot::smp::AP_BOOT_INFO, cpu::{CpuId, CpuSet, PinCurrentCpu}, cpu_local, miri_println, mm::kspace::KERNEL_BASE_VADDR, sync::SpinLock, task::{scheduler::{exit_current, info::CommonSchedInfo, SCHEDULER}, Task, KERNEL_STACK_SIZE}, trap::{self, IrqLine, TrapFrame}
};

/// Execute a function on other processors.
///
/// The provided function `f` will be executed on all target processors
/// specified by `targets`. It can also be executed on the current processor.
/// The function should be short and non-blocking, as it will be executed in
/// interrupt context with interrupts disabled.
///
/// This function does not block until all the target processors acknowledges
/// the interrupt. So if any of the target processors disables IRQs for too
/// long that the controller cannot queue them, the function will not be
/// executed.
///
/// The function `f` will be executed asynchronously on the target processors.
/// However if called on the current processor, it will be synchronous.
pub fn inter_processor_call(targets: &CpuSet, f: fn()) {
    let irq_guard = trap::disable_local();
    let this_cpu_id = irq_guard.current_cpu();
    let irq_num = INTER_PROCESSOR_CALL_IRQ.get().unwrap().num();

    let mut call_on_self = false;
    for cpu_id in targets.iter() {
        if cpu_id == this_cpu_id {
            call_on_self = true;
            continue;
        }
        CALL_QUEUES.get_on_cpu(cpu_id).lock().push_back(f);
    }
    for cpu_id in targets.iter() {
        if cpu_id == this_cpu_id {
            continue;
        }
        // SAFETY: It is safe to send inter processor call IPI to other CPUs.
        unsafe {
            crate::arch::irq::send_ipi(cpu_id, irq_num);
        }
    }
    if call_on_self {
        // Execute the function synchronously.
        f();
    }
}

static INTER_PROCESSOR_CALL_IRQ: Once<IrqLine> = Once::new();

cpu_local! {
    static CALL_QUEUES: SpinLock<VecDeque<fn()>> = SpinLock::new(VecDeque::new());
}

fn do_inter_processor_call(_trapframe: &TrapFrame) {
    // TODO: in interrupt context, disabling interrupts is not necessary.
    let preempt_guard = trap::disable_local();
    let cur_cpu = preempt_guard.current_cpu();

    let mut queue = CALL_QUEUES.get_on_cpu(cur_cpu).lock();
    while let Some(f) = queue.pop_front() {
        log::trace!(
            "Performing inter-processor call to {:#?} on CPU {:#?}",
            f,
            cur_cpu
        );
        f();
    }
}

pub(super) fn init() {
    let mut irq = IrqLine::alloc().unwrap();
    irq.on_active(do_inter_processor_call);
    INTER_PROCESSOR_CALL_IRQ.call_once(|| irq);
}

pub(super) fn init2() {
    if !SCHEDULER.is_completed() {
        crate::task::scheduler::fifo_scheduler::init();
    }

    let ap_boot_info = super::boot::smp::AP_BOOT_INFO.get().unwrap();
    let boot_stack = &ap_boot_info.boot_stack_array;

    unsafe {
        kern_miri_init_ap(1, ap_init_task, 0, &*(0x1000 as *const Task), boot_stack.end_paddr() + KERNEL_BASE_VADDR, boot_stack.size());
    }
}

fn ap_init_task(_temp: usize) {
    unsafe {
        crate::cpu::set_this_cpu_id(1);
        crate::cpu::local::init_on_ap(1);
    }

    // let ap_boot_info = AP_BOOT_INFO.get().unwrap();
    // ap_boot_info
    //     .per_ap_info
    //     .get(&1)
    //     .unwrap()
    //     .is_started
    //     .store(true, Ordering::Release);
    
    let task1 = move || {
        crate::miri_println!("ap init");

        loop {
            crate::task::Task::yield_now();
        }
    };

    let task1 = alloc::sync::Arc::new(
        crate::task::TaskOptions::new(task1)
            .data(())
            .build()
            .unwrap(),
    );

    task1.cpu().set_if_is_none(CpuId::try_from(1).unwrap());
    task1.run();

    exit_current();
}