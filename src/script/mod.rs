use std::collections::{VecDeque, HashMap};
use std::cell::RefCell;
use failure_derive::*;
use itertools::Itertools;
use crate::rom::{Rom, Map, loc::Dma};

pub mod datatype;
pub mod bc;
mod globals;
pub mod parse;

use datatype::*;
use parse::{ast::*, Unparse};
use bc::Bytecode;

struct DecompEnv<'a> {
    scope: &'a mut Scope,
    rom: &'a mut Rom,
    declarations: &'a mut Vec<Declaration>,
    dma: &'a Dma,
}

pub fn decompile_map(map: Map, rom: &mut Rom) -> Result<String, Error> {
    let mut scope = Scope::new();

    // Bring global methods into scope
    for (ptr, name, ty) in &*globals::METHODS {
        scope.insert_ptr(*ptr, name.to_string(), ty.clone());
    }

    let mut declarations = Vec::new();

    let (main_loc, main_bc) = map.main_fun;
    scope.insert_ptr(main_loc.into(), "main".to_string(), DataType::Fun(vec![]));

    let mut env = DecompEnv {
        scope: &mut scope,
        rom,
        declarations: &mut declarations,
        dma: &map.dma,
    };

    decompile_fun("main".to_string(), main_bc, &mut env)?;

    // In-place transformations
    declarations.dedup_by_key(|declaration| match declaration {
        Declaration::Fun { name, .. } => name.clone(),
    });

    // Unparse everything
    Ok(declarations
        .into_iter()
        .rev()
        .map(|declaration| declaration.unparse(&scope))
        .join("\n\n"))
}

fn decompile_fun(name: String, bc: Bytecode, env: &mut DecompEnv) -> Result<(), Error> {
    env.scope.push();

    println!("decompiling function: {}", name);

    let mut block = bc.decompile(env.scope)?;

    fix_call_arg_capture(&mut block, env.scope)?;
    infer_datatypes(&mut block, env.scope)?;

    // Search for more structs we have to decompile, if any
    for (vaddr, name, datatype) in env.scope.pop().unwrap() {
        if let Some(vaddr) = vaddr {
            match datatype {
                DataType::Fun(args) => {
                    env.scope.insert_ptr(
                        vaddr,
                        name.clone(),
                        DataType::Fun(args),
                    );

                    println!("reading bytecode: {}", name);
                    env.rom.seek(env.dma.loc_at_vaddr(vaddr));
                    decompile_fun(name, Bytecode::read(env.rom), env)?;
                },

                DataType::Asm(args) => {
                    env.scope.insert_ptr(
                        vaddr,
                        name,
                        DataType::Asm(args),
                    );

                    // TODO
                },

                _ => (),
            }
        }
    }

    env.declarations.push(Declaration::Fun {
        name:      IdentifierOrPointer::Identifier(Identifier(name)),
        arguments: Vec::new(), // TODO: lookup
        block,
    });

    Ok(())
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
            // Only functions capture - asm methods take args normally.
            if let Some((_, DataType::Fun(argument_types))) = method.lookup(scope) {
                assert_eq!(arguments.len(), 0);

                for (n, _) in argument_types.iter().enumerate() {
                    // TODO: see if FunFlags should be captured if the arg type
                    //       is DataType::Bool

                    let name = format!("{}_{:X}", globals::FUNWORD_STR, n);

                    arguments.push(RefCell::new(Expression::Identifier(Identifier(name))));
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

/// Performs a single type inference pass. Replaces 'any' declarations and their
/// respective scope mappings if their types can be inferred.
fn infer_datatypes(block: &mut Vec<Statement>, mut scope: &mut Scope) -> Result<(), Error> {
    let mut made_inferences = true;

    // This works like a bubble sort -- keep inferring types until we can't.
    while made_inferences {
        made_inferences = false;

        // We only insert inferred types into scope after the interator
        // finishes, because we perform lookups in there and the borrow checker
        // would scream at us for mutating it while we had an immutable ref.
        let mut inferred: Vec<(Option<u32>, String, DataType)> = Vec::new();

        // We iterate in reverse so we can figure out the types before we see their
        // declaration statement (once we do see it, we update its type).
        for stmt in block.iter_mut().rev() {
            match stmt {
                // Update var declarations with inferred types.
                Statement::VarDeclare { datatype, identifier: Identifier(name), expression } => {
                    match scope.lookup_name_depth(&name, 0) {
                        Some(inferred_datatype) => match datatype.replace(DataType::Any) {
                            // User has left it up to the compiler to infer the
                            // type, so lets do that.
                            DataType::Any => {
                                datatype.replace(inferred_datatype.clone());

                                // Update expression literal to the inferred
                                // type, if any.
                                if let Some(expression) = expression {
                                    if let Some((ptr, name, ty)) =
                                        update_literal(expression, inferred_datatype) {
                                        inferred.push((Some(ptr), name, ty));
                                    }
                                }
                            },

                            datatype_inner => if inferred_datatype.clone() == datatype_inner {
                                // Ok, put `datatype_inner` back.
                                datatype.replace(datatype_inner);
                            } else {
                                // User declared the type but we inferred its
                                // use as some other type.
                                println!("warning: {}", Error::VarDeclareTypeMismatch {
                                    identifier:        name.clone(),
                                    declared_datatype: datatype_inner,
                                    inferred_datatype: inferred_datatype.clone(),
                                });
                            },
                        },

                        // The variable is declared here but isn't in the current
                        // scope, so add it to the scope after this pass.
                        None => inferred.push((None, name.clone(), match expression {
                            Some(expression) => expression.borrow().infer_datatype(&scope),
                            None             => DataType::Any,
                        })),
                    }
                },

                Statement::VarAssign { identifier: Identifier(name), expression, .. } => {
                    match scope.lookup_name(name) {
                        // We only need to infer Any (i.e. unknown) types.
                        Some(DataType::Any) | None => inferred.push((
                            None,
                            name.clone(),
                            expression.borrow().infer_datatype(scope)
                        )),

                        // Update float literals
                        Some(DataType::Float) => {
                            if let Some((ptr, name, ty)) = update_literal(expression, &DataType::Float) {
                                inferred.push((Some(ptr), name, ty));
                            }
                        },

                        _ => (),
                    }
                },

                Statement::MethodCall { method, arguments, bc_is_func, .. } => match method.lookup(scope) {
                    Some((_, &DataType::Asm(ref arg_types))) |
                    Some((_, &DataType::Fun(ref arg_types))) => {
                        for (ty, arg) in arg_types.iter().zip(arguments.iter()) {
                            match arg.clone().into_inner() {
                                // Only identifiers influence type inference.
                                Expression::Identifier(Identifier(name)) => {
                                    // We only need to infer Any (i.e. unknown) types.
                                    if let Some(DataType::Any) = scope.lookup_name(&name) {
                                        // Define the inferred type!
                                        inferred.push((None, name.clone(), ty.clone()));
                                    }
                                },

                                // Update literal arg to the type it should be.
                                _ => {
                                    if let Some((ptr, name, ty)) = update_literal(arg, ty) {
                                        inferred.push((Some(ptr), name, ty));
                                    }
                                },
                            }
                        }
                    },

                    _ => if let IdentifierOrPointer::Pointer(ptr) = method {
                        if *ptr & 0xFFFF0000 == 0x80240000 {
                            // Unknown local method pointer; let's define it.

                            let arguments = arguments
                                .iter()
                                .map(|arg_expr| arg_expr.borrow().infer_datatype(scope))
                                .collect();

                            inferred.push((Some(*ptr), name_method_ptr(*ptr), if *bc_is_func {
                                DataType::Fun(arguments)
                            } else {
                                DataType::Asm(arguments)
                            }));
                        }
                    },
                },

                _ => (),
            }

            for mut inner_block in stmt.inner_blocks_mut() {
                infer_datatypes(&mut inner_block, &mut scope)?;
            }
        }

        // Define the inferred types in-scope.
        for (ptr, name, datatype) in inferred.into_iter() {
            if let DataType::Any = datatype {
                // ...why is this even here?
                break
            }

            match scope.insert(ptr, name.clone(), datatype.clone()) {
                Some(DataType::Any) => made_inferences = true,
                Some(old_datatype) => {
                    if datatype != old_datatype {
                        // We accidentally replaced a previous definition.
                        // Don't do that; undo the insertion.
                        println!("infer avoided replacing {} ({} -> {})",
                             name, old_datatype, datatype);
                        scope.insert(ptr, name, old_datatype);
                    }
                },
                None => made_inferences = true,
            }
        }
    }

    Ok(())
}

/// Replaces a given expression node to `datatype` if it is a literal. Returns
/// a generated (ptr, name type) expected to be added to scope if the expression
/// is a raw pointer int literal, updating it to an identifier.
#[must_use]
fn update_literal(expr: &RefCell<Expression>, datatype: &DataType) -> Option<(u32, String, DataType)> {
    match datatype {
        DataType::Bool => {
            if let Expression::LiteralInt(v) = expr.clone().into_inner() {
                expr.replace(Expression::LiteralBool(v == 1));
            }

            None
        },

        DataType::Float => {
            if let Expression::LiteralInt(u) = expr.clone().into_inner() {
                expr.replace(if u == 0 {
                    Expression::LiteralFloat(0.0)
                } else {
                    let s: i32 = unsafe { std::mem::transmute(u) };
                    Expression::LiteralFloat(((s + 230000000) as f32) / 1024.0)
                });
            }

            None
        },

        DataType::Fun(_) | DataType::Asm(_) => {
            if let Expression::LiteralInt(ptr) = expr.clone().into_inner() {
                if ptr & 0xFFFF0000 == 0x80240000 {
                    let name = name_method_ptr(ptr);

                    expr.replace(Expression::Identifier(Identifier(name.clone())));
                    Some((ptr, name, datatype.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        },

        _ => None,
    }
}

fn name_method_ptr(ptr: u32) -> String {
    format!("dump_{:04X}", ptr & 0x0000FFFF)
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed to decompile bytecode: {}", _0)]
    BytecodeDecompile(#[fail(cause)] bc::Error),

    #[fail(display = "variable '{}' declared as {} but is used as {}",
        identifier, declared_datatype, inferred_datatype)]
    VarDeclareTypeMismatch {
        identifier:    String,
        declared_datatype: DataType,
        inferred_datatype: DataType,
    },
}

impl From<bc::Error> for Error {
    fn from(error: bc::Error) -> Error {
        Error::BytecodeDecompile(error)
    }
}

/// A priority-queue mapping of (u32 -> String -> DataType); i.e. Scope provides
/// lookups of pointer-to-name and name-to-datatype, preferring the current
/// scope (see `push` and `pop`) when performing lookups.
#[derive(Debug)]
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

    /// Removes all the mappings in the current scope and returns them, if any.
    pub fn pop(&mut self) -> Option<Vec<(Option<u32>, String, DataType)>> {
        if let Some((ptr_map, mut type_map)) = self.layers.pop_front() {
            let mut flat = Vec::new();

            for (ptr, name) in ptr_map.into_iter() {
                let datatype = type_map.remove(&name).unwrap();
                flat.push((Some(ptr), name, datatype))
            }

            for (name, datatype) in type_map.into_iter() {
                flat.push((None, name, datatype))
            }

            Some(flat)
        } else {
            None
        }
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

    pub fn insert(&mut self, ptr: Option<u32>, name: String, datatype: DataType) -> Option<DataType> {
        if let Some(ptr) = ptr {
            self.insert_ptr(ptr, name, datatype)
        } else{
            self.insert_name(name, datatype)
        }
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

    /// Looks-up the datatype associated with a given name to a given depth.
    /// For example, providing a max depth of 0 would only search the current
    /// scope's mapping.
    pub fn lookup_name_depth(&self, name: &str, max_depth: usize) -> Option<&DataType> {
        for layer in self.layers.iter().take(max_depth + 1) {
            if let Some(datatype) = layer.1.get(name) {
                return Some(datatype);
            }
        }

        None
    }
}
