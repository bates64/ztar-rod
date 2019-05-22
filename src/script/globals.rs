use lazy_static::lazy_static;
use super::DataType::{self, *};

lazy_static! {
    pub static ref METHODS: [(u32, &'static str, DataType); 10] = [

        (0x80285960, "enter_walk", Fun(vec![ Fun(vec![]) ])),

        (0x802C9288, "model_set_vis", Asm(vec![ Int, Bool ])),
        (0x802C9308, "model_set_vis", Asm(vec![ Int, Bool ])), // identicial

        (0x802CA6C0, "cam_set_flag2", Asm(vec![ Int, Int ])),
        (0x802CA774, "cam_set_flag80", Asm(vec![ Int, Int ])),
        (0x802CA828, "cam_set_perspective", Asm(vec![ Int, Int, Int, Int, Int ])),
        (0x802CAB18, "cam_set_viewport", Asm(vec![ Int, Int, Int, Int, Int ])),
        (0x802CAD98, "cam_set_bg_color", Asm(vec![ Int, Int, Int, Int ])),
        (0x802CB680, "cam_set_flag4", Asm(vec![ Int, Int ])),

        (0x802D9700, "set_sprite_shading", Asm(vec![ Int ])),

    ];
}
