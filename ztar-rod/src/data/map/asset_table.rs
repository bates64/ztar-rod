use itertools::Itertools;
use png::HasParameters;
use std::convert::TryInto;
use std::fs::{self, File};

use super::shape::Shape;
use crate::data::color::Palette;
use crate::data::yay0;
use crate::mod_dir::ModDir;
use crate::rom::*;

fn asset_table_addr(rom: &Rom) -> u32 {
    match rom.region {
        Region::Japan => 0x1E00020,
        Region::America => 0x1E40020,
        Region::Europe => 0x2600020,
    }
}

pub struct AssetTable {
    assets: Vec<Asset>,
}

impl AssetTable {
    pub fn dump(self, mod_dir: &ModDir) -> Result<(), std::io::Error> {
        fs::write(
            mod_dir.asset_table(),
            self.assets
                .iter()
                .map(|asset| format!("{}", asset.name))
                .join("\n"),
        )?;

        for asset in self.assets {
            match asset.data {
                AssetData::Background {
                    width,
                    height,
                    raster,
                    palette,
                } => {
                    println!("dumping background: {}", asset.name);

                    let filename = mod_dir.background(asset.name.as_str());
                    let file = File::create(filename)?;

                    let mut encoder = png::Encoder::new(file, u32::from(width), u32::from(height));
                    encoder
                        .set(png::ColorType::Indexed)
                        .set(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().unwrap();

                    writer
                        .write_chunk(png::chunk::PLTE, &palette.1.rgb()[..])
                        .unwrap();
                    writer
                        .write_chunk(png::chunk::tRNS, &palette.1.alpha()[..])
                        .unwrap();
                    writer.write_image_data(&raster.1).unwrap();
                }

                AssetData::Shape { shape } => {
                    // TODO
                }

                AssetData::Unknown { bytes } => {
                    fs::write(mod_dir.built_asset(asset.name.as_str()), &bytes)?;
                }
            }
        }

        Ok(())
    }
}

impl RomRead for AssetTable {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> {
        let mut assets = Vec::new();

        for i in 0..1033 {
            rom.file
                .seek(SeekFrom::Start(u64::from(asset_table_addr(rom) + i * 28)))?;
            assets.push(Asset::read(rom)?);
        }

        Ok(AssetTable { assets })
    }
}

pub struct Asset {
    name: AsciiString,
    data_offset: u32,
    compressed_size: u32,
    decompressed_size: u32,
    data: AssetData,
}

pub enum AssetData {
    Background {
        width: u16,
        height: u16,
        raster: (u32, Vec<u8>),
        palette: (u32, Palette),
    },

    Shape {
        shape: Shape,
    },

    Unknown {
        bytes: Vec<u8>,
    },
}

impl RomRead for Asset {
    fn read(rom: &mut Rom) -> Result<Self, ReadError> {
        // TODO yay0 decompression

        let name = AsciiString::read_len(rom, 16)?;
        let data_offset = u32::read(rom)?;
        let compressed_size = u32::read(rom)?;
        let decompressed_size = u32::read(rom)?;

        let bytes = {
            rom.file.seek(SeekFrom::Start(u64::from(
                asset_table_addr(rom) + data_offset,
            )))?;

            // If the data is compressed, uncompress it.
            if u32::read(rom)? == yay0::MAGIC {
                println!("decompressing asset: {}", name);
                let yay0_size = u32::read(rom)?;
                assert_eq!(yay0_size, decompressed_size);

                rom.file.seek(SeekFrom::Start(u64::from(
                    asset_table_addr(rom) + data_offset,
                )))?;
                let mut buf = vec![0u8; decompressed_size as usize];
                rom.file.read_exact(&mut buf).or(Err(ReadError::Eof))?;

                yay0::decompress(&buf)
            } else {
                rom.file.seek(SeekFrom::Start(u64::from(
                    asset_table_addr(rom) + data_offset,
                )))?;
                let mut buf = vec![0u8; decompressed_size as usize];
                rom.file.read_exact(&mut buf).or(Err(ReadError::Eof))?;
                buf
            }
        };

        Ok(Asset {
            data: if name.as_str().ends_with("_bg") {
                // Image header
                let raster_addr = read_u32(&bytes[0..4]).wrapping_add(0x7FE00000);
                let palette_addr = read_u32(&bytes[4..8]).wrapping_add(0x7FE00000);
                // bytes[8..12]?
                let width = read_u16(&bytes[12..14]);
                let height = read_u16(&bytes[14..16]);

                AssetData::Background {
                    width,
                    height,
                    raster: (raster_addr, {
                        // 8bpp
                        let raster_addr = raster_addr as usize;
                        let raster_len = width as usize * height as usize;
                        bytes[raster_addr..raster_addr + raster_len].to_vec()
                    }),
                    palette: (palette_addr, {
                        let palette_addr = palette_addr as usize;

                        // 256-color RGBA16
                        Palette::from_rgba16(&bytes[palette_addr..palette_addr + 256 * 2])
                    }),
                }
            } else if name.as_str().ends_with("_shape") {
                AssetData::Shape {
                    shape: Shape::parse(bytes)?,
                }
            } else {
                AssetData::Unknown { bytes }
            },

            name,
            data_offset,
            compressed_size,
            decompressed_size,
        })
    }
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from_be_bytes(
        bytes
            .try_into()
            .expect("unexpected end-of-data while reading u16"),
    )
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(
        bytes
            .try_into()
            .expect("unexpected end-of-data while reading u32"),
    )
}
