use super::DataType::{self, *};

// TODO: use lazy_static?

pub fn generate() -> Vec<(u32, &'static str, DataType)> {
    vec![

        (0x80285960, "enter_walk", Fun(vec![ Fun(vec![]) ])),

        (0x802D9700, "use_sprite_shading", Asm(vec![ Int ])),

    ]
}
