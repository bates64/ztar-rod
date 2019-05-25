use std::fs;
use itertools::Itertools;
use crate::rom::*;
use crate::mod_dir::ModDir;
use crate::data::yay0;

fn asset_table_addr(rom: &Rom) -> u32 {
    match rom.region {
        Region::Japan   => 0x1E00020,
        Region::America => 0x1E40020,
        Region::Europe  => 0x2600020,
    }
}

pub struct AssetTable {
    assets: Vec<Asset>,
}

impl AssetTable {
    pub fn dump(&self, mod_dir: &ModDir) -> Result<(), std::io::Error> {
        fs::write(mod_dir.asset_table(), self.assets
            .iter()
            .map(|asset| format!("{}", asset.name))
            .join("\n"))

        // TODO: write *.bin files?
    }
}

impl RomRead for AssetTable {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> {
        let mut assets = Vec::new();

        for i in 0..1033 {
            rom.file.seek(SeekFrom::Start(u64::from(asset_table_addr(rom) + i * 28)))?;
            assets.push(Asset::read(rom)?);
        }

        Ok(AssetTable { assets })
    }
}

pub struct Asset {
    name:              AsciiString,
    data_offset:       u32,
    compressed_size:   u32,
    decompressed_size: u32,
    data:              Option<Vec<u8>>,
}

impl RomRead for Asset {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> {
        // TODO yay0 decompression

        let name              = AsciiString::read_len(rom, 16)?;
        let data_offset       = u32::read(rom)?;
        let compressed_size   = u32::read(rom)?;
        let decompressed_size = u32::read(rom)?;

        let data = {
            rom.file.seek(SeekFrom::Start(u64::from(asset_table_addr(rom) + data_offset)))?;

            // If the data is compressed, uncompress it.
            Some(if u32::from_be_bytes(*b"yay0") == u32::read(rom)? {
                let yay0_size = u32::read(rom)?;
                assert_eq!(yay0_size, decompressed_size);

                let mut buf = vec![0u8; decompressed_size as usize];
                rom.file.read_exact(&mut buf).or(Err(ReadError::Eof))?;

                yay0::decompress(&buf)
            } else {
                let mut buf = vec![0u8; decompressed_size as usize];
                rom.file.read_exact(&mut buf).or(Err(ReadError::Eof))?;
                buf
            })
        };

        Ok(Asset {
            name, data_offset, compressed_size, decompressed_size, data,
        })
    }
}
