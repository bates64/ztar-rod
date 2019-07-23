use std::fmt::{self, Display, Formatter};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum DataType {
    Any,
    Int,
    Float,
    Bool,
    Arr(Box<DataType>),
    Fun(Vec<DataType>),
    Asm(Vec<DataType>),
    // TODO: custom structs and enums
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use DataType::*;
        match self {
            Any       => write!(f, "any"),
            Int       => write!(f, "int"),
            Float     => write!(f, "float"),
            Bool      => write!(f, "bool"),
            Arr(item) => write!(f, "[{}]", item),
            Fun(args) => write!(f, "fun({})", join(args, ", ")),
            Asm(args) => write!(f, "asm({})", join(args, ", ")),
        }
    }
}

fn join<T: Display>(slice: &[T], sep: &str) -> String {
    slice
        .iter()
        .map(|item| format!("{}", item))
        .join(sep)
}
