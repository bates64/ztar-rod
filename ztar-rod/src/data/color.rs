use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Palette {
    pub colors: Vec<Color>,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Palette {
    pub fn into_rgba16(self) -> Vec<u8> {
        let mut packed = Vec::with_capacity(self.colors.len() * 2);

        for color in self.colors {
            let short: u16 = color.into_rgba16();

            packed.push(((short & 0xFF00) >> 4) as u8);
            packed.push((short & 0x00FF) as u8);
        }

        packed
    }

    pub fn from_rgba16(packed: &[u8]) -> Palette {
        let mut colors = Vec::with_capacity(packed.len() / 2);

        for (major, minor) in packed.iter().tuples() {
            let major = u16::from(*major);
            let minor = u16::from(*minor);

            let short: u16 = (major << 8) + minor;
            colors.push(Color::from_rgba16(short));
        }

        Palette { colors }
    }

    pub fn rgb(&self) -> Vec<u8> {
        let mut packed = Vec::with_capacity(self.colors.len() * 3);

        for color in &self.colors {
            packed.push(color.r);
            packed.push(color.g);
            packed.push(color.b);
        }

        packed
    }

    pub fn rgba(&self) -> Vec<u8> {
        let mut packed = Vec::with_capacity(self.colors.len() * 4);

        for color in &self.colors {
            packed.push(color.r);
            packed.push(color.g);
            packed.push(color.b);
            packed.push(color.a);
        }

        packed
    }

    pub fn alpha(&self) -> Vec<u8> {
        let mut packed = Vec::with_capacity(self.colors.len());

        for color in &self.colors {
            packed.push(color.a);
        }

        packed
    }
}

impl Color {
    pub fn into_rgba16(self) -> u16 {
        let r = (31.0 * (f32::from(self.r) / 255.0)).trunc() as u16;
        let g = (31.0 * (f32::from(self.g) / 255.0)).trunc() as u16;
        let b = (31.0 * (f32::from(self.b) / 255.0)).trunc() as u16;
        let opaque = self.a > 0x80;

        let mut color = if opaque { 1 } else { 0 };
        color |= (r & 0x1F) << 11;
        color |= (g & 0x1F) << 6;
        color |= (b & 0x1F) << 1;
        color
    }

    pub fn from_rgba16(s: u16) -> Color {
        Color {
            r: (255.0 * (f32::from((s >> 11) & 0x1F) / 31.0)) as u8,
            g: (255.0 * (f32::from((s >> 6) & 0x1F) / 31.0)) as u8,
            b: (255.0 * (f32::from((s >> 1) & 0x1F) / 31.0)) as u8,
            a: if (s & 1) == 1 { 0xFF } else { 0x00 },
        }
    }

    pub fn into_rgba32(self) -> u32 {
        let mut color = u32::from(self.a);
        color |= u32::from(self.r) << 24;
        color |= u32::from(self.g) << 16;
        color |= u32::from(self.b) << 8;
        color
    }

    pub fn from_rgba32(s: u32) -> Color {
        Color {
            r: (s >> 24) as u8,
            g: (s >> 16) as u8,
            b: (s >> 8) as u8,
            a: s as u8,
        }
    }
}
