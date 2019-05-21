/*
use std::collections::HashMap;
use maplit::hashmap;
use lazy_static::lazy_static;
use super::type::{Type::*, Signature, sig};

lazy_static! {
    // TODO: add everything

    pub static ref GAMEBYTES: HashMap<u32, &'static str> = hashmap!{
        0   => "STORY_PROGRESS",
        425 => "MAP_LOCATION",
    };

    pub static ref METHODS: HashMap<u32, (&'static str, Signature)> = hashmap!{
        //// Scripts ////

        0x80285960 => sig!{ "enter_walk" },

        /*

        //// Functions ////

        0x802C9288 => ("model_set_vis", Signature::new(vec![Int, Bool], None)),
        0x802C9308 => ("model_set_vis", Signature::new(vec![Int, Bool], None)), // identical

        0x802CA6C0 => ("cam_set_flag2", Signature::new(vec![Int, Int], None)),
        0x802CA774 => ("cam_set_flag80", Signature::new(vec![Int, Int], None)),
        0x802CA828 => ("cam_set_perspective", Signature::new(vec![Int, Int, Int, Int, Int], None)),
        0x802CAB18 => ("cam_set_viewport", Signature::new(vec![Int, Int, Int, Int, Int], None)),
        0x802CAD98 => ("cam_set_bg_color", Signature::new(vec![Int, Int, Int, Int], None)),
        0x802CB680 => ("cam_set_flag4", Signature::new(vec![Int, Int], None)),

        0x802D9700 => ("use_sprite_shading", Signature::new(vec![Int], None)),

        */
    };
}
*/
