use std::alloc::Layout;
use std::collections::BTreeMap;

use crate::*;
use rustc_abi::Align;
use rustc_middle::ty::Mutability;

pub const PAGE_SIZE: usize = 4096;
/// The total memory size used in OS memory region.
pub const TOTAL_MEM: usize = 32 * 1024 * PAGE_SIZE;

pub const KERNEL_MEM: usize = 4 * 1024 * PAGE_SIZE;

pub const BASE_BEGIN: u64 = 80 * PAGE_SIZE as u64;
pub const STACK_BEGIN: u64 = 1024 * PAGE_SIZE as u64;

pub const MAX_USERSPACE_VADDR: usize = 0x0000_8000_0000_0000 - PAGE_SIZE;

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

pub fn physical_copy(dst: usize, src: usize, len: usize) {
    unsafe {
        let src_ptr = paddr_to_mem(src) as *const u8;
        let dst_ptr = paddr_to_mem(dst) as *mut u8;

        core::ptr::copy(src_ptr, dst_ptr, len);
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
        let mut allocation = Allocation::<Provenance, (), MiriAllocBytes>::from_bytes(
            std::borrow::Cow::Borrowed(buffer), 
            Align::from_bytes(layout.align() as u64).unwrap(), 
            Mutability::Mut);

        let offset = paddr % PAGE_SIZE;
        if let Some(mask_allocation) = PHYS_INIT_MASK.get(&(paddr - offset)) {
            let init_copy = mask_allocation.init_mask().prepare_copy((offset..offset + layout.size()).into());
            allocation.init_mask_apply_copy(init_copy, (0..layout.size()).into(), 1);
        }
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

pub fn free_allocations<'tcx>(this: &mut MiriInterpCx<'tcx>, paddr: usize, count: usize) -> InterpResult<'tcx, ()>{
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
        unsafe {
            if PAGE_STATES[paddr / PAGE_SIZE] == PageState::Unused {
                throw_ub_format!(
                    "Page state UB: Attempting to release an unused page. The paddr is 0x{:x}", page_paddr
                );
                //panic!("Page state UB: current page state has not been allocated, so it can not be deallocated {:x}", page_paddr);
            }
        }
        set_page_state(page_paddr, PageState::Unused);
        remove_init_mask(page_paddr);
    }
    interp_ok(())
}

pub static mut PHYS_INIT_MASK: BTreeMap<usize, Allocation::<Provenance, (), MiriAllocBytes>> = BTreeMap::new();

pub fn insert_init_mask(this: &mut MiriInterpCx<'_>, paddr: usize) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(PAGE_SIZE, 1);
        let mut allocation = create_allocation_at(paddr, layout);
        allocation.write_uninit(this, (0..PAGE_SIZE).into());

        PHYS_INIT_MASK.insert(paddr, allocation);
    }
}

pub fn remove_init_mask(paddr: usize) {
    unsafe {
        let _ = PHYS_INIT_MASK.remove(&paddr);
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
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PageState {
    Unused,
    Untyped,
    Typed{
        page_type: TypedKind,
        type_size: usize
    },
}

#[derive(Clone, Copy, PartialEq, Debug)]
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
        if PAGE_STATES[index] != page_state {
            panic!("Page state UB: current page state is {:?}", PAGE_STATES[index]);
        }
    }
}

pub fn set_page_state(paddr: usize, page_state: PageState) {
    unsafe {
        let index = paddr / PAGE_SIZE;
        PAGE_STATES[index] = page_state;
    }
}