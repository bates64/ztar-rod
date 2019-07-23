// TODO

use crate::data::color::Color;
use std::io::prelude::*;
use std::io::{self, Cursor, SeekFrom};

const BASE_ADDR: u32 = 0x80210000;

#[derive(Debug)]
pub struct Shape {}

#[derive(Debug)]
struct Vertex {
    x: i16,
    y: i16,
    z: i16,

    texture_x: i16,
    texture_y: i16,

    color: Color,
}

impl Shape {
    pub fn parse(data: Vec<u8>) -> io::Result<Shape> {
        let mut data = Cursor::new(data);

        // Header
        let mesh_tree = read_u32(&mut data) - BASE_ADDR;
        let vertex_table = read_u32(&mut data) - BASE_ADDR;
        let model_name_list = read_u32(&mut data) - BASE_ADDR;
        let collider_name_list = read_u32(&mut data) - BASE_ADDR;
        let zone_name_list = read_u32(&mut data).checked_sub(BASE_ADDR);

        Ok(Shape {})
    }
}

fn read_u32(data: &mut Cursor<Vec<u8>>) -> u32 {
    let mut buf = [0u8; 4];
    data.read_exact(&mut buf).unwrap();
    u32::from_be_bytes(buf)
}
