pub use std::io::{prelude::*, SeekFrom};
pub use ascii::*;
use std::fs::File;
use std::fmt;
use failure::{Error, bail};
use failure_derive::*;

//pub mod loc;
//use loc::*;

/// Wrapper struct for reading (and, soon, writing) a ROM file.
pub struct Rom {
    pub file: File,
    pub region: Region,
}

pub enum Region {
    Japan,
    America,
    Europe,
}

impl Region {
    fn from(byte: u8) -> Option<Region> {
        match byte {
            b'J' => Some(Region::Japan),
            b'E' => Some(Region::America),
            b'P' => Some(Region::Europe),
            _    => None,
        }
    }
}

impl Rom {
    pub fn from(mut file: File) -> Result<Rom, Error> {
        {
            file.seek(SeekFrom::Start(0x00)).unwrap();

            let mut ch = [0u8];
            file.read_exact(&mut ch).unwrap();

            if ch[0] != 0x80 {
                bail!("only big-endian roms are supported (.z64) - try using Tool64 to convert");
            }
        }

        Ok(Rom {
            region: {
                file.seek(SeekFrom::Start(0x3E)).unwrap();

                let mut ch = [0u8];
                file.read_exact(&mut ch).unwrap();

                match Region::from(ch[0]) {
                    Some(region) => region,
                    _  => bail!("only JP, USA, and PAL roms are supported"),
                }
            },
            file,
        })
    }

    /*
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
    */
}

pub trait RomRead {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> where Self: Sized;
}

pub trait RomReadLen {
    /// Should always read exactly `len` bytes.
    fn read_len(rom: &mut Rom, len: usize) -> Result<Self, ReadError> where Self: Sized;
}

#[derive(Debug, Fail)]
pub enum ReadError {
    #[fail(display = "unexpected end of file")]
    Eof,

    #[fail(display = "bad ASCII string: {}", _0)]
    BadAscii(#[fail(cause)] ToAsciiCharError),

    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] std::io::Error),
}

impl From<ToAsciiCharError> for ReadError {
    fn from(error: ToAsciiCharError) -> ReadError {
        ReadError::BadAscii(error)
    }
}

impl From<std::io::Error> for ReadError {
    fn from(error: std::io::Error) -> ReadError {
        ReadError::Io(error)
    }
}

#[derive(Clone, Copy)]
pub enum Pointer {
    Address(u32),
    NullPtr,
}

impl fmt::Display for Pointer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pointer::Address(addr) => write!(fmt, "{:#010X}", addr),
            Pointer::NullPtr       => write!(fmt, "nullptr"),
        }
    }
}

impl fmt::Debug for Pointer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pointer::Address(addr) => write!(fmt, "Address({:#010X})", addr),
            Pointer::NullPtr      => write!(fmt, "NullPtr"),
        }
    }
}

impl RomRead for u32 {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> {
        let mut buf = [0u8; 4];
        rom.file.read_exact(&mut buf).or(Err(ReadError::Eof))?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl RomRead for f32 {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> {
        Ok(f32::from_bits(u32::read(rom)?))
    }
}

impl RomRead for Pointer {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> {
        let weird = u32::read(rom)?;

        Ok(if weird == 0 {
            Pointer::NullPtr
        } else {
            // Pointers in the rom are weirdly formatted; we must
            // force an overflow to get the true address from it.
            Pointer::Address(weird.wrapping_add(2_147_333_120))
        })
    }
}

impl RomRead for AsciiString {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> {
        let mut string = AsciiString::new();

        loop {
            let mut ch = [0u8];
            rom.file.read_exact(&mut ch).or(Err(ReadError::Eof))?;

            // Null-terminator.
            if ch[0] == 0 {
                break;
            }

            string.push(AsciiChar::from(ch[0])?);
        }

        Ok(string)
    }
}

impl RomReadLen for AsciiString {
    fn read_len(rom: &mut Rom, len: usize) -> Result<Self, ReadError> {
        let mut string = AsciiString::new();
        let mut terminated = false;

        for _ in 0..len {
            let mut ch = [0u8];
            rom.file.read_exact(&mut ch).or(Err(ReadError::Eof))?;

            // Null-terminator.
            if ch[0] == 0 {
                terminated = true;
            }

            if !terminated {
                string.push(AsciiChar::from(ch[0])?);
            }
        }

        Ok(string)
    }
}
