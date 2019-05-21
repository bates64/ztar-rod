use std::fmt::{self, Display, Formatter};

/// A variable type.
#[derive(Debug, Clone)]
pub enum Type {
    Undefined,
    Int, Float, Bool,
    Thread, Array(Box<Type>),
    Model, Collider,
    Function, Asm,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&match self {
            Type::Undefined => "undefined".to_string(),
            Type::Int       => "int".to_string(),
            Type::Float     => "float".to_string(),
            Type::Bool      => "bool".to_string(),
            Type::Array(t)  => format!("[{}]", t),
            Type::Thread    => "thread".to_string(),
            Type::Model     => "model".to_string(),
            Type::Collider  => "collider".to_string(),
            Type::Function  => "fun".to_string(),
            Type::Asm       => "asm".to_string(),
        })
    }
}

/// A compile-time type hint for a method.
#[derive(Debug, Clone)]
pub enum Hint {

    Asm(Signature),
    Function(Signature),
}

impl Hint {
    pub fn unwrap_sig(self) -> Signature {
        use Hint::*;
        match self {
            Asm(sig)      => sig,
            Function(sig) => sig,
        }
    }
}
