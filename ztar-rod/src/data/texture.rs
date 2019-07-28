use ascii::AsciiString;
use png::{BitDepth, ColorType, HasParameters};
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::fs::File;

use crate::data::color::{Color, Gray, Palette};

pub fn decode_archive(mut data: &[u8]) -> BTreeMap<AsciiString, Texture> {
    let mut archive = BTreeMap::new();

    while data.len() > 0 {
        let (name, texture, length) = Texture::decode(data);
        data = &data[length..];

        archive.insert(name, texture);
    }

    archive
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Texture {
    pub options: Options,

    pub format: Format,
    pub width: u16,
    pub height: u16,

    pub data: Vec<u8>,
    pub palette: Vec<u8>,
}

// TODO: keep mipmaps and aux data

impl Texture {
    pub fn decode(data: &[u8]) -> (AsciiString, Texture, usize) {
        let name = AsciiString::from_ascii(&data[0..32]).unwrap();

        let width = read_u16(&data[34..36]);
        let mut height = read_u16(&data[38..40]);
        let aux_width = read_u16(&data[32..34]);
        let aux_height = read_u16(&data[36..38]);

        let extra_data = match read_u16(&data[40..42]) {
            0 => ExtraData::None,
            1 => ExtraData::Mipmaps,
            2 => {
                height /= 2;
                ExtraData::Aux
            }
            3 => ExtraData::AuxPlus,
            _ => panic!("illegal extra tiles"),
        };

        let combine_mode = match data[42] {
            0x00 => CombineMode::MultiplyColors,
            0x08 => CombineMode::MultiplyColors_,
            0x0d => CombineMode::DifferenceAlpha,
            0x10 => CombineMode::InterpolatedBlend,
            _ => panic!("illegal combine mode"),
        };

        let format: Format = (data[43] & 0x0f, data[44] & 0x0f).try_into().unwrap();
        let aux_format: Option<Format> = (data[43] >> 4, data[44] >> 4).try_into().ok();

        let horizontal_wrap = match data[45] & 0x0f {
            0 => Wrap::Repeat,
            1 => Wrap::Mirror,
            2 => Wrap::Clamp,
            _ => panic!("illegal wrap"),
        };
        let vertical_wrap = match data[46] & 0x0f {
            0 => Wrap::Repeat,
            1 => Wrap::Mirror,
            2 => Wrap::Clamp,
            _ => panic!("illegal wrap"),
        };

        let filter = match data[47] {
            0 => Filter::None,
            2 => Filter::Bilinear,
            _ => panic!("illegal filtering"),
        };

        let data_size = format.data_size(width, height);
        let palette_size = format.palette_size();

        let data_offset = 48;
        let palette_offset;
        let end;

        match extra_data {
            ExtraData::None => {
                palette_offset = data_offset + data_size;
                end = palette_offset + palette_size;
            }

            ExtraData::Mipmaps => {
                let mut width = width / 2;
                let mut height = height / 2;

                let mut len = 0;

                // TODO: less magic
                while format.data_size(width, 2) >= 16 {
                    len += format.data_size(width, height);
                    width /= 2;
                    height /= 2;
                }

                palette_offset = data_offset + data_size + len;
                end = palette_offset + palette_size;
            }

            ExtraData::Aux => {
                palette_offset = data_offset + 2 * data_size;
                end = palette_offset + palette_size;
            }

            ExtraData::AuxPlus => {
                let aux_format = aux_format.unwrap(); // TODO: use Result
                palette_offset = data_offset + data_size;
                end = palette_offset
                    + palette_size
                    + aux_format.data_size(aux_width, aux_height)
                    + aux_format.palette_size();
            }
        }

        (
            name,
            Texture {
                options: Options {
                    combine_mode,
                    horizontal_wrap,
                    vertical_wrap,
                    filter,
                },
                format,
                width,
                height,
                data: data[data_offset..data_offset + data_size].to_vec(),
                palette: data[palette_offset..palette_offset + palette_size].to_vec(),
            },
            end,
        )
    }

    pub fn save(&self, name: &AsciiString) {
        let null_pos = name
            .as_str()
            .bytes()
            .position(|b| b == 0)
            .unwrap_or(name.len());
        let name = &name[0..null_pos];

        let filename = format!("mod/img/tex/{}.png", name);
        println!("dumping texture: {}", name);

        let mut encoder = png::Encoder::new(
            File::create(filename).unwrap(),
            self.width as u32,
            self.height as u32,
        );
        encoder
            .set(self.format.color_type())
            .set(self.format.bit_depth());
        let mut writer = encoder.write_header().unwrap();

        // Metadata
        writer.write_chunk([b's', b'R', b'G', b'B'], &[0]).unwrap();
        writer
            .write_chunk([b's', b'B', b'I', b'T'], self.format.sig_bits())
            .unwrap();

        // Write palette, if applicable
        match self.format {
            Format::CI4 | Format::CI8 => {
                let palette = Palette::from_rgba16(&self.palette);

                writer
                    .write_chunk(png::chunk::PLTE, &palette.rgb())
                    .unwrap();
                writer
                    .write_chunk(png::chunk::tRNS, &palette.alpha())
                    .unwrap();
            }
            _ => (),
        }

        // Write row-reversed data
        let row_size = self.format.data_size(self.width, 1);
        let rows = self.data.chunks(row_size).rev();
        let mut data = Vec::with_capacity(self.data.len() * 4); // over-estimate

        for row in rows {
            match self.format {
                // These formats can be encoded directly as PNG image data.
                Format::I4
                | Format::I8
                | Format::IA16
                | Format::CI4
                | Format::CI8
                | Format::RGBA32 => data.extend(row),

                // These need per-pixel processing.
                Format::IA4 => {
                    for pair in row {
                        let (hi, lo) = Gray::from_ia4(*pair);
                        data.extend(&hi.into_arr());
                        data.extend(&lo.into_arr());
                    }
                }

                Format::IA8 => {
                    for pixel in row {
                        let g = Gray::from_ia8(*pixel);
                        data.extend(&g.into_arr());
                    }
                }

                Format::RGBA16 => {
                    for pixel in row.chunks(2) {
                        let color = Color::from_rgba16(read_u16(pixel));
                        data.extend(&color.into_arr());
                    }
                }
            }
        }

        writer.write_image_data(&data).unwrap();

        std::mem::drop(writer);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Format {
    I4,
    I8,
    IA4,
    IA8,
    IA16,
    CI4,
    CI8,
    RGBA16,
    RGBA32,
    // YUV16,
}

impl Format {
    fn data_size(&self, width: u16, height: u16) -> usize {
        use Format::*;

        let pixels = (width as usize) * (height as usize);

        match self {
            I4 | IA4 | CI4 => pixels / 2,
            I8 | IA8 | CI8 => pixels,
            IA16 | RGBA16 => pixels * 2,
            RGBA32 => pixels * 4,
        }
    }

    fn palette_size(&self) -> usize {
        use Format::*;
        match self {
            I4 | I8 | IA4 | IA8 | IA16 | RGBA16 | RGBA32 => 0,
            CI4 => 32,
            CI8 => 512,
        }
    }

    fn color_type(&self) -> ColorType {
        use Format::*;
        match self {
            I4 | I8 => ColorType::Grayscale,
            IA4 | IA8 | IA16 => ColorType::GrayscaleAlpha,
            CI4 | CI8 => ColorType::Indexed,
            RGBA16 | RGBA32 => ColorType::RGBA,
        }
    }

    fn bit_depth(&self) -> BitDepth {
        use Format::*;
        match self {
            I4 | CI4 => BitDepth::Four,
            I8 | IA4 | IA8 | IA16 | CI8 | RGBA16 | RGBA32 => BitDepth::Eight,
        }
    }

    fn sig_bits(&self) -> &'static [u8] {
        use Format::*;
        match self {
            I4 => &[4],
            I8 => &[8],
            IA4 => &[3, 1],
            IA8 => &[4, 4],
            IA16 => &[8, 8],
            CI4 | CI8 => &[5, 5, 5],
            RGBA16 => &[5, 5, 5, 1],
            RGBA32 => &[8, 8, 8, 8],
        }
    }
}

impl TryFrom<(u8, u8)> for Format {
    type Error = ();

    fn try_from((format, bit_depth): (u8, u8)) -> Result<Format, ()> {
        match (format, bit_depth) {
            (4, 0) => Ok(Format::I4),
            (4, 1) => Ok(Format::I8),
            (3, 0) => Ok(Format::IA4),
            (3, 1) => Ok(Format::IA8),
            (3, 2) => Ok(Format::IA16),
            (2, 0) => Ok(Format::CI4),
            (2, 1) => Ok(Format::CI8),
            (0, 2) => Ok(Format::RGBA16),
            (0, 3) => Ok(Format::RGBA32),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExtraData {
    None,
    Mipmaps,
    Aux,
    AuxPlus,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Options {
    pub combine_mode: CombineMode,
    pub horizontal_wrap: Wrap,
    pub vertical_wrap: Wrap,
    pub filter: Filter,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Filter {
    None = 0,
    Bilinear = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CombineMode {
    MultiplyColors = 0x00,
    MultiplyColors_ = 0x08,
    DifferenceAlpha = 0x0d,
    InterpolatedBlend = 0x10,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Wrap {
    Repeat = 0,
    Mirror = 1,
    Clamp = 2,
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from_be_bytes(
        bytes
            .try_into()
            .expect("unexpected end-of-data while reading u16"),
    )
}
