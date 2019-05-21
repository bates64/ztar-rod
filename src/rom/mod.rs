use std::io::{prelude::*, SeekFrom};
use std::fs::File;
use crate::script::bc::Bytecode;

pub mod loc;
use loc::*;

/// Wrapper struct for reading and writing to a ROM file.
pub struct Rom {
    pub file: File,
}

impl Rom {
    pub fn from(file: File) -> Rom {
        // TODO fail if little endian

        Rom {
            file,
        }
    }

    /// Seeks to the address of the provided Location.
    pub fn seek(&mut self, loc: Location) -> &mut Rom {
        self.file.seek(loc.into()).unwrap();
        self
    }

    /// Seeks ahead `n` bytes.
    pub fn skip(&mut self, n: i64) -> &mut Rom {
        self.file.seek(SeekFrom::Current(n)).unwrap();
        self
    }

    /// Reads a 4-byte unsigned integer.
    pub fn read_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.file.read_exact(&mut buf).expect("Unexpected EOF");
        u32::from_be_bytes(buf)
    }

    /// Reads a 4-byte float.
    pub fn read_f32(&mut self) -> f32 {
        f32::from_bits(self.read_u32())
    }

    /// Reads a pointer (4 bytes); in the ROM these are stored differently to
    /// standard `u32`s. Returns `None` for null pointers.
    pub fn read_ptr(&mut self) -> Option<u32> {
        let weird = self.read_u32();

        if weird == 0 {
            None // Null pointer.
        } else {
            // Pointers in the rom are weirdly formatted; we must
            // force an overflow to get the true address from it.
            Some(weird.wrapping_add(2_147_333_120))
        }
    }

    /// Reads a null-terminated string. Panics if the string is invalid ASCII.
    pub fn read_string(&mut self) -> String {
        let mut chars = Vec::new();

        loop {
            let mut ch = [0u8];
            self.file.read_exact(&mut ch).expect("Unexpected EOF");

            // Null-terminator.
            if ch[0] == 0 {
                break;
            }

            chars.push(ch[0]);
        }

        // ROM strings are ASCII, but UTF-8 is a superset of it so we can use
        // `from_utf8`.
        std::str::from_utf8(&chars).expect("Malformed string").to_string()
    }

    /// Reads an `Area`. Panics if data is missing or malformed.
    pub fn read_area(&mut self) -> Area {
        let map_count = self.read_u32();
        let maps_addr = self.read_ptr().expect("Area has null maplist");
        let name_addr = self.read_ptr().expect("Area has null name");

        Area {
            name: self.seek(Location::new(name_addr)).read_string(),
            maps: {
                let maps_loc = Location::new(maps_addr);
                let mut maps = Vec::with_capacity(map_count as usize);

                for i in 0..map_count {
                    self.seek(maps_loc.add_offset(i * 32));
                    maps.push(self.read_map())
                }

                maps
            },
        }
    }

    /// Reads `Map`. Panics if data is missing or malformed.
    pub fn read_map(&mut self) -> Map {
        let name_addr = self.read_ptr().expect("Map has null name");
        let header_vaddr = self.read_u32();
        let dma = Dma::new(self.read_u32(), self.read_u32(), self.read_u32());
        let bg_name_addr = self.read_ptr();
        let init_asm_vaddr = self.read_u32(); // might be null
        let flags = self.read_u32();

        let header_loc = dma.loc_at_vaddr(header_vaddr);

        Map {
            name: self.seek(Location::new(name_addr)).read_string(),
            dma,
            init_asm: match init_asm_vaddr {
                0 => None,
                _ => Some(dma.loc_at_vaddr(init_asm_vaddr)),
            },
            main_fun: {
                self.seek(header_loc.add_offset(0x10));
                let loc = dma.loc_at_vaddr(self.read_u32());
                self.seek(loc);
                (loc, Bytecode::read(self))
            },
            entrances: {
                self.seek(header_loc.add_offset(0x14));

                let vaddr = self.read_u32();
                let count = self.read_u32();

                self.seek(dma.loc_at_vaddr(vaddr));
                let mut entrances = Vec::new();

                for _ in 0..count {
                    entrances.push(Entrance {
                        x: self.read_f32(),
                        y: self.read_f32(),
                        z: self.read_f32(),
                        yaw: self.read_f32(),
                    })
                }

                entrances
            },
            tattle: MapTattle::from(Location::new(self.seek(header_loc.add_offset(0x3C)).read_u32())),
            background: match bg_name_addr {
                Some(addr) => Some(self.seek(Location::new(addr)).read_string()),
                None => None,
            },
            flags,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Area {
    pub name: String,
    pub maps: Vec<Map>,
}

#[derive(Debug, Clone)]
pub struct Map {
    // Assets (TODO)
    //pub shape: Shape,
    //pub hit: Hit,

    // Areatable
    pub name: String,
    pub background: Option<String>,
    pub flags: u32, // TODO maybe use a bitfield enum for this

    // Header
    pub dma: Dma,
    pub entrances: Vec<Entrance>,
    pub tattle: MapTattle,
    pub init_asm: Option<Location>, // TODO
    pub main_fun: (Location, Bytecode),
}

/// Map entrance position.
#[derive(Debug, Clone)]
pub struct Entrance {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
}

/// Goombario's tattle text. Can be dynamically returned by an asm method.
#[derive(Debug, Clone)]
pub enum MapTattle {
    Id(u16, u16),
    Asm(Location),
}

impl From<Location> for MapTattle {
    fn from(loc: Location) -> MapTattle {
        let addr = loc.offset;
        if is_vaddr(addr) {
            MapTattle::Asm(loc)
        } else {
            let major = (addr & 0xFFFF_0000) >> 16;
            let minor = addr & 0x0000_FFFF;
            MapTattle::Id(major as u16, minor as u16)
        }
    }
}
