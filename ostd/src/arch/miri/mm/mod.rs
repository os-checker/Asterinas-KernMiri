// SPDX-License-Identifier: MPL-2.0

use alloc::fmt;
use core::ops::Range;

use crate::{
    mm::{
        page_prop::{CachePolicy, PageFlags, PageProperty, PrivilegedPageFlags as PrivFlags},
        page_table::PageTableEntryTrait,
        Paddr, PagingConstsTrait, PagingLevel, Vaddr, PAGE_SIZE,
    },
    Pod,
};

use super::{kern_miri_copy, kern_miri_log};

pub(crate) const NR_ENTRIES_PER_PAGE: usize = 512;

#[derive(Clone, Debug, Default)]
pub struct PagingConsts {}

impl PagingConstsTrait for PagingConsts {
    const BASE_PAGE_SIZE: usize = 4096;
    const NR_LEVELS: PagingLevel = 4;
    const ADDRESS_WIDTH: usize = 48;
    const HIGHEST_TRANSLATION_LEVEL: PagingLevel = 4;
    const PTE_SIZE: usize = core::mem::size_of::<PageTableEntry>();
}

bitflags::bitflags! {
    #[derive(Pod)]
    #[repr(C)]
    /// Possible flags for a page table entry.
    pub struct PageTableFlags: usize {
        /// Specifies whether the mapped frame or page table is valid.
        const VALID =           1 << 0;
        /// Controls whether reads to the mapped frames are allowed.
        const READABLE =        1 << 1;
        /// Controls whether writes to the mapped frames are allowed.
        const WRITABLE =        1 << 2;
        /// Controls whether execution code in the mapped frames are allowed.
        const EXECUTABLE =      1 << 3;
        /// Controls whether accesses from userspace (i.e. U-mode) are permitted.
        const USER =            1 << 4;
        /// Indicates that the mapping is present in all address spaces, so it isn't flushed from
        /// the TLB on an address space switch.
        const GLOBAL =          1 << 5;

        const UNCACHEABLE =     1 << 6;
        /// In level 2 or 3 it indicates that it map to a huge page.
        /// In level 1, it is the PAT (page attribute table) bit.
        /// We use this bit in level 1, 2 and 3 to indicate that this entry is
        /// "valid". For levels above 3, `PRESENT` is used for "valid".
        const HUGE =            1 << 7;

        const WRITE_THROUGH =     1 << 8;
    }
}

pub(crate) fn tlb_flush_addr(vaddr: Vaddr) {
}

pub(crate) fn tlb_flush_addr_range(range: &Range<Vaddr>) {
    for vaddr in range.clone().step_by(PAGE_SIZE) {
        tlb_flush_addr(vaddr);
    }
}

pub(crate) fn tlb_flush_all_excluding_global() {

}

pub(crate) fn tlb_flush_all_including_global() {

}

#[derive(Clone, Copy, Pod, Default)]
#[repr(C)]
pub struct PageTableEntry(usize);

extern "Rust" {
    /// Activates the page table with the root physical address `root_paddr` 
    /// as the currently used page table. When `miri` performs this operation, 
    /// it will recursively traverse the page table nodes starting from `root_paddr`. 
    /// If any intermediate node points to a page that does not have 
    /// the expected page state for a page table node, it will be treated as UB.
    fn kern_miri_set_root_page_table(root_paddr: Paddr);

    /// Obtains the root_paddr of the currently active page table.
    fn kern_miri_get_root_page_table() -> Paddr;
}

/// Activate the given level 4 page table.
///
/// "satp" register doesn't have a field that encodes the cache policy,
/// so `_root_pt_cache` is ignored.
///
/// # Safety
///
/// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
/// changing the page mapping.
pub unsafe fn activate_page_table(root_paddr: Paddr, _root_pt_cache: CachePolicy) {
    kern_miri_set_root_page_table(root_paddr);
}

pub fn current_page_table_paddr() -> Paddr {
    unsafe { kern_miri_get_root_page_table()}
}

impl PageTableEntry {
    const PHYS_ADDR_MASK: usize = 0xF_FFFF_FFFF_F000 | 1 << 7;

    fn new_paddr(paddr: Paddr) -> Self {
        Self(paddr)
    }
}

/// Parse a bit-flag bits `val` in the representation of `from` to `to` in bits.
macro_rules! parse_flags {
    ($val:expr, $from:expr, $to:expr) => {
        ($val as usize & $from.bits() as usize) >> $from.bits().ilog2() << $to.bits().ilog2()
    };
}

impl PageTableEntryTrait for PageTableEntry {
    fn is_present(&self) -> bool {
        self.0 & PageTableFlags::VALID.bits() != 0 || self.0 & PageTableFlags::HUGE.bits() != 0
    }

    fn new_page(paddr: Paddr, _level: PagingLevel, prop: PageProperty) -> Self {
        let flags = PageTableFlags::HUGE.bits();
        let mut pte = Self::new_paddr(paddr | flags);
        pte.set_prop(prop);
        pte
    }

    fn new_pt(paddr: Paddr) -> Self {
        let pte = Self::new_paddr(paddr);
        PageTableEntry(pte.0 | PageTableFlags::VALID.bits())
    }

    fn paddr(&self) -> Paddr {
        self.0 & 0xF_FFFF_FFFF_F000
    }

    fn prop(&self) -> PageProperty {
        let flags = parse_flags!(self.0, PageTableFlags::READABLE, PageFlags::R)
            | parse_flags!(self.0, PageTableFlags::WRITABLE, PageFlags::W)
            | parse_flags!(self.0, PageTableFlags::EXECUTABLE, PageFlags::X);
        let priv_flags = parse_flags!(self.0, PageTableFlags::USER, PrivFlags::USER)
            | parse_flags!(self.0, PageTableFlags::GLOBAL, PrivFlags::GLOBAL);

        let cache = if self.0 & PageTableFlags::UNCACHEABLE.bits() != 0 {
            CachePolicy::Uncacheable
        } else if self.0 & PageTableFlags::WRITE_THROUGH.bits() != 0 {
            CachePolicy::Writethrough
        } else {
            CachePolicy::Writeback
        };

        PageProperty {
            flags: PageFlags::from_bits(flags as u8).unwrap(),
            cache,
            priv_flags: PrivFlags::from_bits(priv_flags as u8).unwrap(),
        }
    }

    fn set_prop(&mut self, prop: PageProperty) {
        let mut flags = PageTableFlags::VALID.bits()
        | parse_flags!(prop.flags.bits(), PageFlags::R, PageTableFlags::READABLE)
        | parse_flags!(prop.flags.bits(), PageFlags::W, PageTableFlags::WRITABLE)
        | parse_flags!(prop.flags.bits(), PageFlags::X, PageTableFlags::EXECUTABLE)
        | parse_flags!(
            prop.priv_flags.bits(),
            PrivFlags::USER,
            PageTableFlags::USER
        )
        | parse_flags!(
            prop.priv_flags.bits(),
            PrivFlags::GLOBAL,
            PageTableFlags::GLOBAL
        );

        match prop.cache {
            CachePolicy::Writeback => (),
            CachePolicy::Writethrough => {
                // Currently, Asterinas uses `Uncacheable` for I/O memory.
                flags |= PageTableFlags::WRITE_THROUGH.bits()
            }
            CachePolicy::Uncacheable => {
                // Currently, Asterinas uses `Uncacheable` for I/O memory.
                flags |= PageTableFlags::UNCACHEABLE.bits()
            }
            _ => panic!("unsupported cache policy"),
        }

        self.0 = (self.0 & Self::PHYS_ADDR_MASK) | flags;
    }

    fn is_last(&self, level: PagingLevel) -> bool {
        self.0 & PageTableFlags::HUGE.bits() != 0
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("raw", &format_args!("{:#x}", self.0))
            .field("paddr", &format_args!("{:#x}", self.paddr()))
            .field("present", &self.is_present())
            .field(
                "flags",
                &false,
            )
            .field("prop", &self.prop())
            .finish()
    }
}

pub(crate) fn __memcpy_fallible(dst: *mut u8, src: *const u8, size: usize) -> usize {
    unsafe {
        kern_miri_copy(dst as usize, src as usize, size);
    }
    //unsafe { core::ptr::copy(src, dst, size) };
    0
}

pub(crate) fn __memset_fallible(dst: *mut u8, value: u8, size: usize) -> usize {
    unsafe { core::ptr::write_bytes(dst, value, size) };
    0
}
