pub use super::super::datatype::DataType;

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
        datatype:    DataType,
        identifiers: Vec<Identifier>,
        expression:  Option<Expression>,
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

#[derive(Debug, Clone)]
pub struct Identifier(pub String);
