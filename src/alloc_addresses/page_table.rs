use std::cell::RefCell;
use std::collections::BTreeMap;

use physical_mem::{paddr_to_mem, retype_pages_at, TOTAL_MEM};

use crate::physical_mem::{PAGE_SIZE, PHYSICAL_MEM};
use crate::*;


pub const NR_LEVELS: usize = 4;
pub const ADDRESS_WIDTH: usize = 48;
pub const PTE_SIZE: usize = 8;

const PTE_PER_PAGE: usize = PAGE_SIZE / PTE_SIZE;
const PTE_INDEX_BITS: usize = PTE_PER_PAGE.ilog2() as usize;

pub const LINEAR_MAPPING_BASE_VADDR: usize = 0xffff_8000_0000_0000;
pub const LINEAR_MAPPING_END_VADDR: usize = 0xffff_c000_0000_0000;

pub const KERNEL_CODE_BASE_VADDR: usize = 0xffff_ffff_8000_0000;
pub const KERNEL_CODE_END_VADDR: usize = 0xffff_ffff_ffff_0000;

pub const BOOT_PT_PADDR: usize = 0x1000;
pub const BOOT_PT_PDPT_PADDR: usize = 0x2000;

pub const BOOT_PT_PD_0G_1G: usize = 0x3000;

pub const BOOT_PT_PT_ADDR: usize = 0x5000;


pub unsafe fn init_boot_pt(this: &mut MiriInterpCx<'_>) -> PageTable {
    let page_table = PageTable::new(BOOT_PT_PADDR);

    *(paddr_to_mem(BOOT_PT_PADDR) as *mut usize) = BOOT_PT_PDPT_PADDR;
    *(paddr_to_mem(BOOT_PT_PADDR) as *mut usize).add(0x100) = BOOT_PT_PDPT_PADDR;
    *(paddr_to_mem(BOOT_PT_PADDR) as *mut usize).add(0x1ff) = BOOT_PT_PDPT_PADDR;

    *(paddr_to_mem(BOOT_PT_PDPT_PADDR) as *mut usize) = BOOT_PT_PD_0G_1G;
    // *(paddr_to_mem(BOOT_PT_PDPT_PADDR) as *mut usize).add(1) = BOOT_PT_PD_1G_2G;
    // *(paddr_to_mem(BOOT_PT_PDPT_PADDR) as *mut usize).add(2) = BOOT_PT_PD_2G_3G;
    // *(paddr_to_mem(BOOT_PT_PDPT_PADDR) as *mut usize).add(3) = BOOT_PT_PD_3G_4G;

    *(paddr_to_mem(BOOT_PT_PDPT_PADDR) as *mut usize).add(0x1fe) = BOOT_PT_PD_0G_1G;
    // *(paddr_to_mem(BOOT_PT_PDPT_PADDR) as *mut usize).add(0x1ff) = BOOT_PT_PD_1G_2G;

    let mut target_addr = 0;
    let mut pd_addr = BOOT_PT_PD_0G_1G;
    let mut pt_addr = BOOT_PT_PT_ADDR;

    retype_pages_at(this, BOOT_PT_PADDR, 3, PTE_SIZE, physical_mem::TypedKind::PageTable).unwrap();
    retype_pages_at(this, BOOT_PT_PT_ADDR, 64, PTE_SIZE, physical_mem::TypedKind::PageTable).unwrap();

    let page_num = TOTAL_MEM / (PAGE_SIZE * PTE_PER_PAGE);

    for i in 0..page_num {
        *(paddr_to_mem(pd_addr) as *mut usize) = pt_addr;
        
        for k in 0..PAGE_SIZE / PTE_SIZE {
            *((paddr_to_mem(pt_addr) as *mut usize).add(k)) = target_addr;
            target_addr += PAGE_SIZE;
        }

        pd_addr += PTE_SIZE;
        pt_addr += PAGE_SIZE;
    }

    page_table
}

#[derive(Debug)]
pub struct PageTable {
    root_paddr: usize,
    typed_page_paddr_to_vaddr: RefCell<BTreeMap<usize, usize>>,
}

impl PageTable {
    const LEVEL_MASK: usize = PTE_PER_PAGE - 1;
    const HUGE_BIT_MASK: usize = 1 << 7;
 
    /// The index of a VA's PTE in a page table node at the given level.
    const fn pte_index(va: usize, level: usize) -> usize {
        va >> (PAGE_SIZE.ilog2() as usize + PTE_INDEX_BITS * (level - 1))
            & Self::LEVEL_MASK
    }

    /// Creates a new `PageTable` where `root_paddr` is `paddr`.
    /// Used when OS invoking `kern_miri_set_root_page_table`.
    pub fn new(paddr: usize) -> Self {
        Self {
            root_paddr: paddr,
            typed_page_paddr_to_vaddr: RefCell::new(BTreeMap::new()),
        }
    }
    /// Gets the root paddr of this `PageTable`.
    /// Used when OS invoking `kern_miri_get_root_page_table`
    pub fn root_paddr(&self) -> usize {
        self.root_paddr
    }

    pub fn page_walk(&self, vaddr: usize) -> Option<usize> {
        let mut current_paddr = self.root_paddr;
        let mut current_level = NR_LEVELS;

        while current_level >= 1 {
            let index = Self::pte_index(vaddr, current_level);

            let page_table_entry = unsafe {
                let pte_paddr = (current_paddr as *const usize).add(index) as usize;
                *(paddr_to_mem(pte_paddr) as *const usize)
            };

            const PTE_MASK: usize = 0xF_FFFF_FFFF_F000;
            current_paddr = page_table_entry & PTE_MASK;
            current_level -= 1;

            if page_table_entry & Self::HUGE_BIT_MASK > 0 {
                break;
            }
        }

        let page_offset = vaddr & ((PAGE_SIZE << (current_level * PTE_INDEX_BITS)) - 1) ;
        Some(current_paddr + page_offset)
    }

    pub fn paddr_to_vaddr(&self, paddr: usize) -> Option<usize> {
        let map = self.typed_page_paddr_to_vaddr.borrow();
        map.get(&paddr).map(|vaddr| *vaddr)
    }
}