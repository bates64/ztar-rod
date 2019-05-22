use std::cell::RefCell;
pub use super::super::{Scope, datatype::DataType};

pub trait InnerBlocks {
    fn inner_blocks(&self) -> Vec<&Vec<Statement>>;
    fn inner_blocks_mut(&mut self) -> Vec<&mut Vec<Statement>>;
}

#[derive(Debug, Clone)]
pub struct Script(Vec<Declaration>);

#[derive(Debug, Clone)]
pub enum Declaration {
    Fun {
        name:      IdentifierOrPointer,
        arguments: Vec<(Identifier, DataType)>,
        block:     Vec<Statement>,
    },
}

impl InnerBlocks for Declaration {
    fn inner_blocks(&self) -> Vec<&Vec<Statement>> {
        match self {
            Declaration::Fun { block, .. } => vec![block],
        }
    }

    fn inner_blocks_mut(&mut self) -> Vec<&mut Vec<Statement>> {
        match self {
            Declaration::Fun { block, .. } => vec![block],
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return,

    Label { name: String },
    Goto  { label_name: String },

    VarAssign {
        identifier: Identifier,
        expression: Expression,
    },

    VarDeclare {
        datatype:   RefCell<DataType>, // interior mutability needed for type inference
        identifier: Identifier,
        expression: Option<Expression>,
    },

    MethodCall {
        method:    IdentifierOrPointer,
        arguments: Vec<Expression>,
        threading: MethodThreading,
    },

    Wait { time: Expression, unit: TimeUnit },

    If {
        condition:   Expression,
        block_true:  Vec<Statement>,
        block_false: Vec<Statement>,
    },

    Switch {
        expression: Expression,
        cases:      Vec<(Case, Vec<Statement>)>,
    },
}

impl InnerBlocks for Statement {
    fn inner_blocks(&self) -> Vec<&Vec<Statement>> {
        match self {
            Statement::If { block_true, block_false, .. } =>
                vec![block_true, block_false],

            Statement::Switch { cases, .. } =>
                cases.iter().map(|(_, block)| block).collect(),

            _ => vec![],
        }
    }

    fn inner_blocks_mut(&mut self) -> Vec<&mut Vec<Statement>> {
        match self {
            Statement::If { block_true, block_false, .. } =>
                vec![block_true, block_false],

            Statement::Switch { cases, .. } =>
                cases.iter_mut().map(|(_, block)| block).collect(),

            _ => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    LiteralInt(u32),
    LiteralFloat(f32),
    LiteralBool(bool),

    Identifier(Identifier),
    ArrayIndex(Identifier, u8),

    Operation {
        lhs: Box<Expression>,
        op:  Operator,
        rhs: Box<Expression>,
    },
}

impl Expression {
    pub fn infer_datatype(&self, scope: &Scope) -> DataType {
        match self {
            Expression::LiteralInt(_)   => DataType::Int,
            Expression::LiteralFloat(_) => DataType::Float,
            Expression::LiteralBool(_)  => DataType::Bool,

            Expression::Identifier(Identifier(name)) => match scope.lookup_name(name) {
                Some(datatype) => datatype.clone(),
                None           => DataType::Any,
            },

            Expression::ArrayIndex(Identifier(name), _) => match scope.lookup_name(name) {
                Some(datatype) => match datatype {
                    DataType::Arr(item_ty) => *item_ty.clone(),
                    _                      => DataType::Any, // ???
                },
                None           => DataType::Any,
            },

            Expression::Operation { lhs, op, .. } => match op {
                Operator::Add |
                Operator::Sub |
                Operator::Mul |
                Operator::Div |
                Operator::Mod => lhs.infer_datatype(scope),

                Operator::Eq  |
                Operator::Ne  |
                Operator::Gt  |
                Operator::Lt  |
                Operator::Gte |
                Operator::Lte |
                Operator::BitAndZ  |
                Operator::BitAndNz |
                Operator::And |
                Operator::Or  |
                Operator::Not => DataType::Bool,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MethodThreading {
    No,                 // method()
    Yes,                // thread method()
    Assign(Identifier), // var = thread method()
}

#[derive(Debug, Clone)]
pub enum TimeUnit {
    Frames,
    Seconds,
}

#[derive(Debug, Clone)]
pub enum Case {
    Default,
    Test {
        operator: Operator,
        against:  Expression,
    },
}

#[derive(Debug, Clone)]
pub enum Operator {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,

    // Logic
    Eq, Ne, Gt, Lt, Gte, Lte,
    BitAndZ, BitAndNz,
    And, Or, Not,
}

#[derive(Debug, Clone)]
pub enum IdentifierOrPointer {
    Identifier(Identifier),
    Pointer(u32),
}

impl IdentifierOrPointer {
    pub fn lookup<'a>(&'a self, scope: &'a super::super::Scope) -> Option<(&'a str, &'a DataType)> {
        match self {
            IdentifierOrPointer::Identifier(Identifier(name)) =>
                scope.lookup_name(name).and_then(|ty| Some((name.as_str(), ty))),

             // Look-up the pointer - if it has a name, use the name instead
            IdentifierOrPointer::Pointer(ptr) => match scope.lookup_ptr(*ptr) {
                Some(name) =>
                    scope.lookup_name(name)
                    .and_then(|ty| Some((name, ty))),
                None       => None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Identifier(pub String);
