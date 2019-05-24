use lazy_static::lazy_static;
use super::DataType::{self, *};

pub static GAMEBYTE_STR:   &'static str = "gamebyte";
pub static AREABYTE_STR:   &'static str = "areabyte";
pub static MAPWORD_STR:    &'static str = "mapvar";
pub static FUNWORD_STR:    &'static str = "var";

pub static GAMEFLAG_STR:   &'static str = "gameflag";
pub static AREAFLAG_STR:   &'static str = "areaflag";
pub static MAPFLAG_STR:    &'static str = "mapflag";
pub static FUNFLAG_STR:    &'static str = "flag";

pub static FLAGARRAY_STR:  &'static str = "flags";
pub static ARRAY_STR:      &'static str = "array";

lazy_static! {
    pub static ref METHODS: [(u32, &'static str, DataType); 23] = [

        (0x80285960, "enter_walk", Fun(vec![ Fun(vec![]) ])),
        (0x80285CF4, "exit_walk", Fun(vec![])), // see set_exit_heading

        (0x80285CB0, "enter_save_point", Fun(vec![])), // for loadtype == 1

        (0x80285DD4, "enter_door", Fun(vec![ Int, Int/*unused?*/, Int, Int ])),
        (0x80285DAC, "exit_door", Fun(vec![ Int, Int, Int, Int ])),

        (0x80285E74, "enter_double_door", Fun(vec![ Int, Int/*unused?*/, Int, Int ])),
        (0x80285E4C, "exit_double_door", Fun(vec![ Int, Int, Int, Int ])),

        (0x802C8B60, "model_translate", Asm(vec![ Int, Float, Float, Float ])),
        (0x802C8C64, "model_rotate", Asm(vec![ Int, Int, Int, Int, Int ])),
        (0x802C8D88, "model_scale", Asm(vec![ Int, Int, Int, Int ])),
        (0x802C8F28, "model_clone", Asm(vec![ Int, Int ])),
        // this returns on var0, var1, var2 but they are not passed in as args...
        // TODO consider making a wrapper script?
        //(0x802C8F80, "model_get_center", Asm(vec![ Int ])),
        (0x802C9000, "model_set_panning", Asm(vec![ Int, Int ])),
        (0x802C9000, "model_enable_flag10", Asm(vec![ Int, Bool ])),
        // 0x802C90FC
        // 0x802C91A4
        (0x802C9208, "model_enable_panning", Asm(vec![ Int, Bool ])),
        (0x802C9288, "model_enable", Asm(vec![ Int, Bool ])),
        (0x802C9308, "model_enable", Asm(vec![ Int, Bool ])), // identicial
        // 0x802C9364 set_tex_pan
        // 0x802C9428
        // 0x802C94A0

        (0x802CA6C0, "cam_enable_flag2", Asm(vec![ Int, Bool ])),
        (0x802CA774, "cam_enable_flag80", Asm(vec![ Int, Bool ])),
        (0x802CA828, "cam_set_perspective", Asm(vec![ Int, Int, Int, Int, Int ])),
        (0x802CAB18, "cam_set_viewport", Asm(vec![ Int, Int, Int, Int, Int ])),
        (0x802CAD98, "cam_set_bg_color", Asm(vec![ Int, Int, Int, Int ])),
        (0x802CB680, "cam_enable_flag4", Asm(vec![ Int, Bool ])),

        (0x802D9700, "set_sprite_shading", Asm(vec![ Int ])),

    ];
}
