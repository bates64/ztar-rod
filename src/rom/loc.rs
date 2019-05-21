use std::fmt::{self, Display, Formatter};
use std::io::SeekFrom;

/// Returns true if the provided address is a runtime RAM 'virtual address'.
pub fn is_vaddr(addr: u32) -> bool {
    addr & 0xFF000000 == 0x80000000
}

/// Holds a base address and offset. Useful for pointer calculations.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Location {
    /// Base address.
    pub base: u32,

    /// Address offset.
    pub offset: u32,
}

impl Location {
    pub fn new(base_addr: u32) -> Location {
        Location {
            base: base_addr,
            offset: 0,
        }
    }

    pub fn add_offset(self, offset: u32) -> Location {
        Location {
            base: self.base,
            offset: self.offset + offset,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "0x{:08X}+{:04X}", self.base, self.offset)
    }
}

impl Into<u32> for Location {
    fn into(self) -> u32 {
        self.base + self.offset
    }
}

impl Into<SeekFrom> for Location {
    fn into(self) -> SeekFrom {
        SeekFrom::Start(u64::from(self.base + self.offset))
    }
}

/// Dynamically-allocated memory; the data at `start`..`end` will be copied to
/// address `dest` at some point during runtime.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dma {
    /// Starting address (ROM).
    pub start: u32,

    /// Ending address (ROM).
    pub end: u32,

    /// Destination virtual-address (RAM).
    pub dest: u32,
}

impl Dma {
    /// Panics if `dest` is not a virtual address (see `is_vaddr`).
    pub fn new(start: u32, end: u32, dest: u32) -> Dma {
        if !is_vaddr(dest) {
            panic!("Dma::new called with non-vaddr destination");
        }

        Dma { start, end, dest }
    }

    pub fn loc_at_vaddr(&self, vaddr: u32) -> Location {
        Location {
            base: self.start,
            offset: vaddr - self.dest,
        }
    }

    pub fn loc_at_offset(&self, offset: u32) -> Location {
        Location {
            base: self.start,
            offset,
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }
}
