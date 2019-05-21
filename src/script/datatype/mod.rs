use std::fmt::{self, Display, Formatter};

mod priomap;
pub use priomap::PrioMap;

#[derive(Debug, Clone)]
pub enum DataType {
    Any,
    Int,
    Float,
    Bool,
    Fun(Vec<DataType>),
    Asm(Vec<DataType>),
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use DataType::*;
        match self {
            Any       => write!(f, "any"),
            Int       => write!(f, "int"),
            Float     => write!(f, "float"),
            Bool      => write!(f, "bool"),
            Fun(args) => write!(f, "fun({})", join(args, ", ")),
            Asm(args) => write!(f, "asm({})", join(args, ", ")),
        }
    }
}

fn join<T: Display>(slice: &[T], sep: &str) -> String {
    slice
        .iter()
        .map(|item| format!("{}", item))
        .collect::<Vec<String>>()
        .join(sep)
}
