use std::fmt::Write;
use failure_derive::*;
use crate::rom::{Rom, Map};

pub mod datatype;
pub mod bc;
mod globals;
pub mod parse;

use datatype::*;
use Error::*;

pub fn decompile_map(map: Map, _rom: &mut Rom) -> Result<String, Error> {
    let mut scope = PrioMap::new();
    let mut out   = String::new();

    // Bring global methods into scope
    for (loc, _, ty) in globals::generate() {
        scope.insert(loc, ty);
    }

    {
        let (loc, bc) = map.main_fun;

        // Main function takes no arguments
        scope.insert(loc, DataType::Fun(vec![]));

        // Functions get their own scope
        scope.push();

        // Decode the function bytecode
        // TODO: type inference pass followed by source reconstruction
        writeln!(out, "/*\n{:#?}\n*/", bc
            .decompile()
            .or_else(|err| Err(BytecodeDecompile(err)))?
        ).unwrap();

        // Pop the function scope; we don't need it anymore
        println!("{:?}", scope.pop());
    }

    Ok(out)
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed to decompile bytecode: {}", _0)]
    BytecodeDecompile(#[fail(cause)] bc::Error),
}
