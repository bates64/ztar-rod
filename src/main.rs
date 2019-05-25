#![allow(clippy::unreadable_literal)]

mod rom;
mod data;
//mod script;
mod mod_dir;

use std::path::Path;
use std::fs::File;
use rom::*;
use mod_dir::ModDir;
use data::map::asset_table::AssetTable;
//use dir::ModDir;

fn main() {
    static ROM_JAPAN: &'static str   = "Mario Story (J) [!].z64";
    static ROM_AMERICA: &'static str = "Paper Mario (U) [!].z64";
    static ROM_EUROPE: &'static str  = "Paper Mario (Europe) (En,Fr,De,Es).z64";

    match File::open(ROM_EUROPE) {
        Err(_)  => println!("unable to open rom"),
        Ok(rom) => match dump(rom) {
            Err(error) => println!("{}", error),
            Ok(())     => (),
        }
    }
}

fn dump(rom: File) -> Result<(), failure::Error> {
    let mut rom = Rom::from(rom)?;
    let mod_dir = ModDir::open(Path::new("./mod"));

    mod_dir.reset().unwrap();

    AssetTable::read(&mut rom)?.dump(&mod_dir)?;

    Ok(())
}
