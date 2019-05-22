use std::fmt::Write;
use std::collections::{VecDeque, HashMap};
use failure_derive::*;
use crate::rom::{Rom, Map};

pub mod datatype;
pub mod bc;
mod globals;
pub mod parse;

use datatype::*;
use parse::{ast::*, Unparse};

pub fn decompile_map(map: Map, _rom: &mut Rom) -> Result<String, Error> {
    let mut scope        = Scope::new();
    let mut declarations = Vec::new();

    // Bring global methods into scope
    for (ptr, name, ty) in &*globals::METHODS {
        scope.insert_ptr(*ptr, name.to_string(), ty.clone());
    }

    {
        let (loc, bc) = map.main_fun;

        // Main function takes no arguments
        scope.insert_ptr(loc.into(), "main".to_string(), DataType::Fun(vec![]));

        // Decompile the bytecode
        let mut decl = Declaration::Fun {
            name:      IdentifierOrPointer::Pointer(loc.into()),
            arguments: Vec::new(),
            block:     bc.decompile()?,
        };

        // TODO: type inference here

        // TODO: decompile pointers within

        // TODO: type inference here

        for mut block in decl.inner_blocks_mut() {
            fix_call_arg_capture(&mut block, &scope)?
        }

        // TODO: type inference here

        declarations.push(decl);
    }

    // Unparse everything
    let mut out = String::new();

    for declaration in declarations.into_iter() {
        writeln!(out, "{}", declaration.unparse(&scope)).unwrap();
    }

    Ok(out)
}

/// Paper Mario function calls capture their environment -- that is, they take
/// every single FunWord/FunFlag as an argument by default. This fixes method
/// calls to do just that depending on the function signature defined in the
/// given Scope. This fn expects that all function signatures are correctly
/// defined in-scope.
///
/// For example, entry_walk takes a single argument, so the following:
///
///     callback = myscript
///     entry_walk()
///
/// Would be transformed into:
///
///     callback = myscript
///     entry_walk(callback)
///
/// Note that this transformation should only be applied to decompiled ASTs, not
/// those the user gives us; this should be a missing-method-arg error.
fn fix_call_arg_capture(block: &mut Vec<Statement>, scope: &Scope) -> Result<(), Error> {
    for stmt in block.iter_mut() {
        if let Statement::MethodCall { method, arguments, .. } = stmt {
            // Lookup the method signature.
            let datatype = match method {
                IdentifierOrPointer::Identifier(Identifier(name)) =>
                    scope.lookup_name(name),

                 // Look-up the pointer - if it has a name, use the name instead
                IdentifierOrPointer::Pointer(ptr) => match scope.lookup_ptr(*ptr) {
                    Some(name) => scope.lookup_name(name),
                    None       => None,
                },
            };

            // Only functions capture - asm methods take args normally.
            if let Some(DataType::Fun(argument_types)) = datatype {
                assert_eq!(arguments.len(), 0);

                for (n, _) in argument_types.iter().enumerate() {
                    // TODO: see if FunFlags should be captured if the arg type
                    //       is DataType::Bool

                    let name = format!("{}_{:X}", globals::FUNWORD_STR, n);

                    arguments.push(Expression::Identifier(Identifier(name)));
                }
            }
        }

        // Fix inner blocks, too.
        for mut inner_block in stmt.inner_blocks_mut() {
            fix_call_arg_capture(&mut inner_block, &scope)?;
        }
    }

    Ok(())
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
