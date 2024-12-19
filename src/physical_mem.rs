use std::alloc::Layout;

use crate::*;
use rustc_abi::Align;
use rustc_middle::ty::Mutability;

pub const PAGE_SIZE: usize = 4096;
/// The total memory size used in OS memory region.
pub const TOTAL_MEM: usize = 32 * 1024 * PAGE_SIZE;

pub const KERNEL_MEM: usize = 4 * 1024 * PAGE_SIZE;

pub const BASE_BEGIN: u64 = 80 * PAGE_SIZE as u64;
pub const STACK_BEGIN: u64 = 1024 * PAGE_SIZE as u64;

/// The pointer to the simulated physical memory.
pub static mut PHYSICAL_MEM: *mut u8 = std::ptr::null_mut();

pub fn init_miri_physical_mem() {
    unsafe {
        PHYSICAL_MEM = std::alloc::alloc_zeroed(Layout::from_size_align(TOTAL_MEM, PAGE_SIZE).unwrap());

        for i in 0..KERNEL_MEM / PAGE_SIZE {
            PAGE_STATES[i] = PageState::Typed { page_type: TypedKind::Interpreter, type_size: PAGE_SIZE };
        }
    }
}

/// Creates an `Allocation` at `paddr` with `layout`.
/// 
/// The `paddr` is the physical address in the OS. This method will
/// put the backend bytes of created allocation in the corresponding
/// position of the simulated physical memory.
pub fn create_allocation_at(paddr: usize, layout: Layout) 
-> Allocation<Provenance, (), MiriAllocBytes>{
    unsafe {
        let start = paddr_to_mem(paddr);
        let buffer = std::slice::from_raw_parts(start, layout.size());
        let allocation = Allocation::<Provenance, (), MiriAllocBytes>::from_bytes(
            std::borrow::Cow::Borrowed(buffer), 
            Align::from_bytes(layout.align() as u64).unwrap(), 
            Mutability::Mut);

        allocation
    }
}

pub fn retype_pages_at<'tcx>(this: &mut MiriInterpCx<'tcx>, paddr: usize, count: usize, type_size: usize, page_type: TypedKind) -> InterpResult<'tcx, ()> {
    let mut alloc_map = this.memory.alloc_map().0.borrow_mut();
    let mut global_state = this.machine.alloc_addresses.borrow_mut();
    //let kind = rustc_const_eval::interpret::MemoryKind::Machine(MiriMemoryKind::Kernel);
    
    for page_index in 0..count {
        let page_paddr = paddr + PAGE_SIZE * page_index;
        set_page_state(page_paddr, PageState::Typed { page_type, type_size});
        // for index in 0..PAGE_SIZE / type_size {
        //     let alloc_id = this.tcx.reserve_alloc_id();
        //     let actual_paddr = page_paddr + index * type_size;
            
        //     let allocation = {
        //         let allocation = create_allocation_at(actual_paddr, Layout::from_size_align(type_size, type_size).unwrap());
        //         let extra = MiriMachine::init_alloc_extra(this, alloc_id, kind, allocation.size(), allocation.align)?;
        //         allocation.with_extra(extra)
        //     };

        //     alloc_map.insert(alloc_id, Box::new((kind, allocation)));
        //     global_state.set_address(alloc_id, actual_paddr);
        // }
    }

    interp_ok(())
}

pub fn free_allocations(this: &mut MiriInterpCx<'_>, paddr: usize, count: usize) {
    let mut alloc_map = this.memory.alloc_map().0.borrow_mut();
    let mut global_state = this.machine.alloc_addresses.borrow_mut();

    for page_index in 0..count {
        let page_paddr = paddr + PAGE_SIZE * page_index;
        let page_info = unsafe {
            PAGE_STATES[paddr / PAGE_SIZE]
        };

        if let PageState::Typed { page_type, type_size } = page_info {
            for index in 0..PAGE_SIZE / type_size {
                let actual_paddr = page_paddr + index * type_size;
                let pos = global_state.int_to_ptr_map.binary_search_by_key(&(actual_paddr as u64), |(addr, _)| *addr);
                if let Ok(pos) = pos {
                    let dead_id = global_state.int_to_ptr_map[pos].1;
                    global_state.int_to_ptr_map.remove(pos);
                    global_state.exposed.remove(&dead_id);
                    global_state.base_addr.remove(&dead_id);
                    alloc_map.remove(&dead_id);
                }
            }
        }
        set_page_state(page_paddr, PageState::Unused);
        
    }

}

/// Convert a physical address to a pointer that point to
/// the corresponding position of the simulated physical memory.
pub fn paddr_to_mem(paddr: usize) -> *mut u8 {
    unsafe {
        PHYSICAL_MEM.add(paddr)
    }
}

/// Additional state settings for the physical pages maintained by Miri. 
/// Initially, all pages are set to `Unused`.
/// PageState transformation: 
/// `Unused` --allocate--> `Untyped` --retype--> `Typed`.
/// `Untyped`/`Typed` --deallocate--> `Unused`.
#[derive(Clone, Copy, PartialEq)]
pub enum PageState {
    Unused,
    Untyped,
    Typed{
        page_type: TypedKind,
        type_size: usize
    },
}

#[derive(Clone, Copy, PartialEq)]
pub enum TypedKind {
    Slab = 1,
    PageTable = 2,
    Stack = 3,
    Interpreter = 4,
}

impl TypedKind {
    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            1 => Some(TypedKind::Slab),
            2 => Some(TypedKind::PageTable),
            3 => Some(TypedKind::Stack),
            4 => Some(TypedKind::Interpreter),
            _ => None,
        }
    }
}

pub static mut PAGE_STATES: [PageState; TOTAL_MEM / PAGE_SIZE] = [PageState::Unused; TOTAL_MEM / PAGE_SIZE];

pub fn check_page_state(paddr: usize, page_state: PageState) {
    unsafe {
        let index = paddr / PAGE_SIZE;
        assert!(PAGE_STATES[index] == page_state);
    }
}

pub fn set_page_state(paddr: usize, page_state: PageState) {
    unsafe {
        let index = paddr / PAGE_SIZE;
        PAGE_STATES[index] = page_state;
    }
}