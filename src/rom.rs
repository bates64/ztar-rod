use std::convert::TryInto;
use std::io::{prelude::*, SeekFrom};
use crate::script::bc::{Bytecode, Operation, Symbol};

pub struct Rom<F: Read + Write + Seek> {
    pub file: F,
}

impl<F: Read + Write + Seek> Rom<F> {
    pub fn from(file: F) -> Rom<F> {
        // TODO fail if little endian

        Rom {
            file: file,
        }
    }

    pub fn seek(&mut self, loc: Location) -> &mut Rom<F> {
        self.file.seek(loc.into()).unwrap();
        self
    }

    pub fn skip(&mut self, bytes: i64) -> &mut Rom<F> {
        self.file.seek(SeekFrom::Current(bytes)).unwrap();
        self
    }

    pub fn read_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.file.read_exact(&mut buf).expect("Unexpected EOF");
        u32::from_be_bytes(buf)
    }

    pub fn read_f32(&mut self) -> f32 {
        f32::from_bits(self.read_u32())
    }

    pub fn read_addr(&mut self) -> Option<u32> {
        let weird = self.read_u32();

        if weird == 0 {
            None // Null pointer.
        } else {
            // Pointers in the rom are weirdly formatted; we must
            // force an overflow to get the true address from it.
            Some(weird.wrapping_add(2_147_333_120))
        }
    }

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

        std::str::from_utf8(&chars).expect("Malformed string").to_string()
    }

    pub fn read_area(&mut self) -> Area {
        let map_count = self.read_u32();
        let maps_addr = self.read_addr().expect("Area has null maplist");
        let name_addr = self.read_addr().expect("Area has null name");

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

    pub fn read_map(&mut self) -> Map {
        let name_addr = self.read_addr().expect("Map has null name");
        let header_vaddr = self.read_u32();
        let dma = Dma::new(self.read_u32(), self.read_u32(), self.read_u32());
        let bg_name_addr = self.read_addr();
        let init_asm_vaddr = self.read_u32(); // might be null
        let flags = self.read_u32();

        let header_loc = dma.loc_at_vaddr(header_vaddr);

        Map {
            name: self.seek(Location::new(name_addr)).read_string(),
            dma: dma,
            init_asm: match init_asm_vaddr {
                0 => None,
                _ => Some(dma.loc_at_vaddr(init_asm_vaddr)),
            },
            main_func: {
                let loc = dma.loc_at_vaddr(self.seek(header_loc.add_offset(0x10)).read_u32());
                self.seek(loc).read_bytecode(loc.offset)
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
            tattle: Text::from(self.seek(header_loc.add_offset(0x3C)).read_u32(), &dma),
            background: match bg_name_addr {
                Some(addr) => Some(self.seek(Location::new(addr)).read_string()),
                None => None,
            },
            flags: flags,
        }
    }

    pub fn read_bytecode(&mut self, offset: u32) -> Bytecode {
        let mut data = Vec::new();

        loop {
            let opcode = self.read_u32() as u8;

            let mut args = Vec::new();
            for _ in 0..self.read_u32() {
                args.push(Symbol::from(self.read_u32()))
            }

            match opcode.try_into() {
                Ok(op) => {
                    data.push((op, args));

                    if let Operation::End = op {
                        return Bytecode {
                            offset: offset,
                            data: data,
                        };
                    }
                },
                Err(_) => panic!("Unknown function opcode: 0x{:02X}", opcode),
            }
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
    pub name: String,
    pub init_asm: Option<Location>,
    pub main_func: Bytecode,
    pub dma: Dma,
    pub entrances: Vec<Entrance>,
    pub tattle: Text,
    //pub data: MapData,
    pub background: Option<String>,
    pub flags: u32, // TODO maybe use a bitfield enum for this
}

#[derive(Debug, Clone)]
pub struct Entrance {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone)]
pub enum Text {
    Id(u16, u16),
    Dynamic(Location), // Function.
}

impl Text {
    pub fn from(u: u32, dma: &Dma) -> Text {
        if is_vaddr(u) {
            Text::Dynamic(dma.loc_at_vaddr(u))
        } else {
            let section = (u & 0xFFFF0000) >> 16;
            let number = u & 0x0000FFFF;
            Text::Id(section as u16, number as u16)
        }
    }
}

pub fn is_vaddr(vaddr: u32) -> bool {
    vaddr & 0xFF000000 == 0x80000000
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Location {
    pub base: u32,
    pub offset: u32,
}

impl Location {
    pub fn new(base_addr: u32) -> Location {
        Location {
            base: base_addr,
            offset: 0,
        }
    }

    pub fn add_offset(&self, offset: u32) -> Location {
        Location {
            base: self.base,
            offset: self.offset + offset,
        }
    }
}

impl Into<SeekFrom> for Location {
    fn into(self) -> SeekFrom {
        SeekFrom::Start((self.base + self.offset) as u64)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dma {
    pub start: u32,
    pub end: u32,
    pub dest: u32,
}

impl Dma {
    pub fn new(start: u32, end: u32, dest: u32) -> Dma {
        Dma {
            start: start,
            end: end,
            dest: dest,
        }
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
            offset: offset,
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }
}
