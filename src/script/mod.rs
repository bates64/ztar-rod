use std::fmt::Write;
use std::collections::{VecDeque, HashMap};
use failure_derive::*;
use crate::rom::{Rom, Map};

pub mod datatype;
pub mod bc;
mod globals;
pub mod parse;

use datatype::*;
use parse::{ast::{Declaration, IdentifierOrPointer}, Unparse};

pub fn decompile_map(map: Map, _rom: &mut Rom) -> Result<String, Error> {
    let mut scope        = Scope::new();
    let mut declarations = Vec::new();

    // Bring global methods into scope
    for (ptr, name, ty) in globals::generate() {
        scope.insert_ptr(ptr, name.to_string(), ty);
    }

    {
        let (loc, bc) = map.main_fun;

        // Main function takes no arguments
        scope.insert_ptr(loc.into(), "main".to_string(), DataType::Fun(vec![]));

        // Decompile the bytecode
        declarations.push(Declaration::Fun {
            name:      IdentifierOrPointer::Pointer(loc.into()),
            arguments: Vec::new(),
            block:     bc.decompile()?,
        })

        // TODO: decompile pointers within
        // TODO: type inference
    }

    // Unparse everything
    let mut out = String::new();

    for declaration in declarations.into_iter() {
        writeln!(out, "{}", declaration.unparse(&scope)).unwrap();
    }

    Ok(out)
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed to decompile bytecode: {}", _0)]
    BytecodeDecompile(#[fail(cause)] bc::Error),
}

impl From<bc::Error> for Error {
    fn from(error: bc::Error) -> Error {
        Error::BytecodeDecompile(error)
    }
}

/// A priority-queue mapping of (u32 -> String -> DataType); i.e. Scope provides
/// lookups of pointer-to-name and name-to-datatype, preferring the current
/// scope (see `push` and `pop`) when performing lookups.
pub struct Scope {
    layers: VecDeque<(HashMap<u32, String>, HashMap<String, DataType>)>,
}

impl Scope {
    /// Creates a new Scope.
    pub fn new() -> Scope {
        let mut scope = Scope { layers: VecDeque::new() };
        scope.push();
        scope
    }

    /// Adds a new mapping on-top of the current scope. Values inserted
    /// will 'shadow' (soft-overwrite) values below with the same key until
    /// this scope is popped.
    pub fn push(&mut self) {
        self.layers.push_front((HashMap::new(), HashMap::new()));
    }

    /// Removes the current scope mapping and returns it, if any.
    pub fn pop(&mut self) -> Option<(HashMap<u32, String>, HashMap<String, DataType>)> {
        self.layers.pop_front()
    }

    /// Inserts a (u32 -> String -> DataType) mapping. If the current scope
    /// already has either key, they are updated and their previous datatype
    /// is returned.
    pub fn insert_ptr(&mut self, ptr: u32, name: String, datatype: DataType) -> Option<DataType> {
        self.layers[0].0.insert(ptr, name.clone());
        self.layers[0].1.insert(name, datatype)
    }

    /// Inserts a (String -> DataType) mapping. If the current scope already has
    /// this name mapped, it is updated and its previous datatype returned.
    pub fn insert_name(&mut self, name: String, datatype: DataType) -> Option<DataType> {
        self.layers[0].1.insert(name, datatype)
    }

    /// Looks-up the name associated with a given pointer.
    pub fn lookup_ptr(&self, ptr: u32) -> Option<&str> {
        for layer in self.layers.iter() {
            if let Some(name) = layer.0.get(&ptr) {
                return Some(name);
            }
        }

        None
    }

    /// Looks-up the datatype associated with a given name.
    pub fn lookup_name(&self, name: &str) -> Option<&DataType> {
        for layer in self.layers.iter() {
            if let Some(datatype) = layer.1.get(name) {
                return Some(datatype);
            }
        }

        None
    }
}
