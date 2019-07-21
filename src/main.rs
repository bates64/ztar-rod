#![allow(clippy::unreadable_literal)]

mod data;
mod rom;
//mod script;
mod mod_dir;

use data::map::asset_table::AssetTable;
use mod_dir::ModDir;
use rom::*;
use std::fs::File;
use std::path::Path;
//use dir::ModDir;

fn main() {
    static ROM_JAPAN: &'static str = "Mario Story (J) [!].z64";
    static ROM_AMERICA: &'static str = "Paper Mario (U) [!].z64";
    static ROM_EUROPE: &'static str = "Paper Mario (Europe) (En,Fr,De,Es).z64";

    match File::open(ROM_AMERICA) {
        Err(_) => println!("unable to open rom"),
        Ok(rom) => match dump(rom) {
            Err(error) => println!("{}", error),
            Ok(()) => (),
        },
    }
}

fn dump(rom: File) -> Result<(), failure::Error> {
    let mut rom = Rom::from(rom)?;
    let mod_dir = ModDir::open(Path::new("./mod"));

    mod_dir.reset().unwrap();

    AssetTable::read(&mut rom)?.dump(&mod_dir)?;

    Ok(())
}
