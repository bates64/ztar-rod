use crate::rom::loc::Location;
use super::DataType::{self, *};

pub fn generate() -> Vec<(Location, &'static str, DataType)> {
    vec![

        (Location::new(0x80285960), "enter_walk", Fun(vec![ Fun(vec![]) ])),

        (Location::new(0x802D9700), "use_sprite_shading", Asm(vec![ Int ])),

    ]
}
