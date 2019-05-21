pub mod ast;

// TODO

/*
use std::convert::{TryInto, TryFrom};
use itertools::Itertools;
use pest::{Parser, iterators::Pair};
use pest_derive::*;
use super::types::*;

/// Error type with associated location; e.g. `Span`. These have a very nice
/// implementation of `std::fmt::Display`, so they're good for user-facing
/// error messages relating to parsing or compilation. See also `bail_at!`.
pub type Error = pest::error::Error<Rule>;

/// Returns an `Error` at the given `Span` with a custom message. Format strings
/// and arguments are accepted.
#[macro_export] macro_rules! bail_at {
    ($span:expr, $str:expr) => {
        return Err(Error::new_from_span(pest::error::ErrorVariant::CustomError {
            message: $str.to_string(),
        }, $span));
    };
    ($span:expr, $fmt:expr, $($arg:tt)*) => {
        return Err(Error::new_from_span(pest::error::ErrorVariant::CustomError {
            message: format!($fmt, $($arg)*),
        }, $span));
    };
}

#[derive(Parser)]
#[grammar = "script/parse/grammar.pest"]
struct ScriptParser;

/// Parses a given string into a u32. Handles hex (0x) and binary (0b) too.
pub fn parse_int(s: &str) -> Result<u32, std::num::ParseIntError> {
    if s.starts_with("0x") {
        u32::from_str_radix(&s[2..], 16)
    } else if s.starts_with("0b") {
        u32::from_str_radix(&s[2..], 2)
    } else {
        s.parse()
    }
}

/// Parsing for `Type`.
impl<'a> TryFrom<Pair<'a, Rule>> for Type {
    type Error = Error;
    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Error> {
        Ok(match pair.as_rule() {
            Rule::ty => match pair.as_str() {
                "int" => Type::Int,
                "float" => Type::Float,
                "bool" => Type::Bool,
                "fun" => Type::Function,
                "asm" => Type::Asm,
                _ => bail_at!(pair.as_span(), "unknown type"),
            },
            _ => bail_at!(pair.as_span(), "expected type"),
        })
    }
}

/// Literal value known at compile-time.
#[derive(Debug, PartialEq)]
pub enum Value {
    Int(u32),
    //Uint(u8)
    Float(f32),
    Bool(bool),
    //Function(Some(Identifier))
    //Asm(Some(Identifier))
}

impl<'a> TryFrom<Pair<'a, Rule>> for Value {
    type Error = Error;
    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Error> {
        Ok(match pair.as_rule() {
            Rule::literal => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    Rule::literal_int => Value::Int(parse_int(pair.as_str()).unwrap()),
                    Rule::literal_float => Value::Float(pair.as_str().parse().unwrap()),
                    Rule::literal_bool => Value::Bool(pair.as_str() == "true"),
                    _ => bail_at!(pair.as_span(), "unimplemented literal"),
                }
            },
            _ => bail_at!(pair.as_span(), "expected literal"),
        })
    }
}

/// Represents a function that can be called in scripts.
#[derive(Debug)]
pub struct Function {
    pub offset:    Option<u32>,
    pub signature: Signature,
    pub stmts:     Vec<Statement>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Function {
    type Error = Error;
    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Error> {
        Ok(match pair.as_rule() {
            Rule::function => {
                let mut pairs = pair.into_inner();

                Function {
                    offset: None,
                    signature: Signature {
                        name: pairs.next().unwrap().as_str().to_string(),
                        arguments: {
                            let mut args = Vec::new();
                            for (id, ty) in pairs.next().unwrap().into_inner().tuples() {
                                args.push((
                                    id.as_str().to_string(),
                                    ty.try_into()?,
                                ));
                            }
                            args
                        },
                    },
                    stmts: collect_stmts(pairs.next().unwrap())?,
                }
            },
            _ => bail_at!(pair.as_span(), "expected function"),
        })
    }
}

impl TryFrom<&str> for Function {
    type Error = Error;
    fn try_from(source: &str) -> Result<Function, Error> {
        let pair = ScriptParser::parse(Rule::function, &source)?
            .next().unwrap();
        Ok(pair.try_into()?)
    }
}

#[derive(Debug)]
pub enum Statement {
    Call(Call),
    //VarDeclare { ident: Identifier, expr: Expression },
    VarAssign  { ident: Identifier, op: char, expr: Expression },

    SleepFrames { time: Expression },
    SleepSecs   { time: Expression },
    Return,
    GotoLabel(String),
    Label(String),

    If {
        cond: Expression,
        if_true: Vec<Statement>,
        if_false: Option<Vec<Statement>>,
    },
    Switch {
        expr: Expression,
        branches: Vec<(Vec<SwitchCase>, Vec<Statement>)>,
    },
}

#[derive(Debug)]
pub enum SwitchCase {
    DefaultCase,
    Eq(Expression),
    Ne(Expression),
    Lt(Expression),
    Gt(Expression),
    Lte(Expression),
    Gte(Expression),
}

fn collect_stmts(pair: Pair<Rule>) -> Result<Vec<Statement>, Error> {
    Ok(match pair.as_rule() {
        Rule::stmt  => vec![pair.try_into()?],
        Rule::stmts => pair.into_inner()
            .map(|pair| pair.try_into())
            .collect::<Result<Vec<_>, _>>()?,
        _ => bail_at!(pair.as_span(), "expected statement(s)"),
    })
}

impl<'a> TryFrom<Pair<'a, Rule>> for Statement {
    type Error = Error;
    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Error> {
        Ok(match pair.as_rule() {
            Rule::stmt => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    // Function or asm call.
                    Rule::call => Statement::Call(pair.try_into()?),

                    /*
                    // Variable declarations include their type followed by their
                    // identifier. They can optionally be followed by an initial value;
                    // if a value is not specified we choose a sensible default.
                    Rule::var_declare => {
                        let mut pairs = pair.into_inner();
                        let ty: Type = pairs.next().unwrap().try_into()?;

                        Statement::VarDeclare {
                            ident: pairs.next().unwrap().try_into()?,
                            expr: if let Some(expr) = pairs.next() {
                                expr.try_into()?
                            } else {
                                Expression::Literal(ty.default_value())
                            }
                        }
                    },
                    */

                    Rule::var_assign => {
                        let mut pairs = pair.into_inner();

                        Statement::VarAssign {
                            ident: pairs.next().unwrap().try_into()?,
                            op: pairs.next().unwrap().as_str().chars().next().unwrap(),
                            expr: pairs.next().unwrap().try_into()?,
                        }
                    },

                    Rule::sleep_stmt => {
                        let mut pairs = pair.into_inner();
                        let expr = pairs.next().unwrap().try_into()?;
                        let time_unit = pairs.next().unwrap();
                        match time_unit.as_str() {
                            "secs" => Statement::SleepSecs   { time: expr },
                            "" => Statement::SleepFrames { time: expr },
                            _ => bail_at!(time_unit.as_span(), "unknown time unit"),
                        }
                    },

                    Rule::return_stmt => Statement::Return,

                    Rule::goto_stmt =>
                        Statement::GotoLabel(pair.into_inner().next().unwrap().as_str().to_string()),
                    Rule::label_stmt =>
                        Statement::Label(pair.into_inner().next().unwrap().as_str().to_string()),

                    // If-else statement. Else-ifs are parsed as nested stmts.
                    Rule::if_stmt => {
                        let mut pairs = pair.into_inner();
                        Statement::If {
                            cond: pairs.next().unwrap().try_into()?,
                            if_true: collect_stmts(pairs.next().unwrap())?,
                            if_false: match pairs.next() {
                                Some(pair) => Some(collect_stmts(pair)?),
                                None => None,
                            },
                        }
                    },

                    // Switch-case statement.
                    Rule::switch_stmt => {
                        let mut pairs = pair.into_inner();
                        let value = pairs.next().unwrap().try_into()?;
                        let mut branches = Vec::new();
                        let mut seen_default = false;

                        for pair in pairs {
                            let mut cases = Vec::new();
                            let mut stmts;

                            // Each branch can have multiple clauses attached,
                            // so we consume all of them.
                            let mut switch_case = pair.into_inner();
                            loop {
                                let clause = switch_case.next().unwrap();

                                // Error if we've already consumed a default
                                // case yet there are clauses after it.
                                if seen_default {
                                    bail_at!(clause.as_span(), "unreachable case as there is a `default` above it");
                                }

                                cases.push(match clause.as_rule() {
                                    Rule::default_case => {
                                        seen_default = true;
                                        SwitchCase::DefaultCase
                                    },
                                    Rule::op => {
                                        let op = clause.into_inner().next().unwrap();
                                        let expr = switch_case.next().unwrap().try_into()?;
                                        match op.as_rule() {
                                            Rule::op_eq  => SwitchCase::Eq(expr),
                                            Rule::op_ne  => SwitchCase::Ne(expr),
                                            Rule::op_lt  => SwitchCase::Lt(expr),
                                            Rule::op_gt  => SwitchCase::Gt(expr),
                                            Rule::op_lte => SwitchCase::Lte(expr),
                                            Rule::op_gte => SwitchCase::Gte(expr),
                                            _ => bail_at!(op.as_span(), "invalid comparison operation"),
                                        }
                                    },
                                    _ => unreachable!(),
                                });

                                // Further cases are nested within the next
                                // pair. If we detect statements instead, we
                                // consume those and end the loop.
                                let next_pair = switch_case.next().unwrap();
                                match next_pair.as_rule() {
                                    Rule::switch_case =>
                                        switch_case = next_pair.into_inner(),
                                    _ => {
                                        stmts = collect_stmts(next_pair)?;
                                        break;
                                    },
                                }
                            }

                            branches.push((cases, stmts));
                        }

                        Statement::Switch {
                            expr: value,
                            branches: branches,
                        }
                    },

                    _ => bail_at!(pair.as_span(), "unimplemented statement"),
                }
            },

            _ => bail_at!(pair.as_span(), "expected statement"),
        })
    }
}

#[derive(Debug)]
pub enum Expression {
    Literal(Value),
    Identifier(Identifier),
    Call(Call),

    // Prefix operators
    Negate(Box<Expression>),

    // Infix operators (see impl TryFrom for precedence/associativity)
    Equal { lhs: Box<Expression>, rhs: Box<Expression> },
    NotEqual { lhs: Box<Expression>, rhs: Box<Expression> },
}

impl<'a> TryFrom<Pair<'a, Rule>> for Expression {
    type Error = Error;
    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Error> {
        use pest::prec_climber::*;

        // note: this is magic

        let climber = PrecClimber::new(vec![
            // == !=
            Operator::new(Rule::op_eq, Assoc::Left) | Operator::new(Rule::op_ne, Assoc::Left),
        ]);

        fn term(pair: Pair<Rule>) -> Result<Expression, Error> {
            Ok(match pair.as_rule() {
                Rule::term => term(pair.into_inner().next().unwrap())?,
                Rule::paren_expr => pair.into_inner().next().unwrap().try_into()?,

                Rule::literal    => Expression::Literal(pair.try_into()?),
                Rule::id         => Expression::Identifier(pair.try_into()?),
                Rule::arr_access => Expression::Identifier(pair.try_into()?),
                Rule::call       => Expression::Call(pair.try_into()?),

                Rule::negate => Expression::Negate(Box::new(term(pair.into_inner().next().unwrap())?)),

                _ => bail_at!(pair.as_span(), "unimplemented term: {}", pair),
            })
        }

        let infix =|lhs: Result<Expression, Error>, op: Pair<Rule>, rhs: Result<Expression, Error>| {
            Ok(match op.as_rule() {
                Rule::op_eq => Expression::Equal {
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                },
                Rule::op_ne => Expression::NotEqual {
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                },
                _ => bail_at!(op.as_span(), "unimplemented operator"),
            })
        };

        match pair.as_rule() {
            Rule::expr => climber.climb(pair.into_inner(), term, infix),
            _ => bail_at!(pair.as_span(), "expected expression: {}", pair),
        }
    }
}

#[derive(Debug)]
pub struct Call {
    target: CallTarget,
    args:   Vec<Expression>,
    thread: bool,
}

#[derive(Debug)]
pub enum CallTarget {
    Known(Identifier),
    Foreign(u32),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Call {
    type Error = Error;
    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Error> {
        Ok(match pair.as_rule() {
            Rule::call => {
                let mut pairs = pair.into_inner();
                let mut is_threaded = false;

                Call {
                    target: {
                        let target = {
                            let pair = pairs.next().unwrap();
                            match pair.as_rule() {
                                Rule::thread => {
                                    is_threaded = true;
                                    pairs.next().unwrap()
                                },
                                _ => pair,
                            }
                        };

                        match target.as_rule() {
                            Rule::id          => CallTarget::Known(target.try_into()?),
                            Rule::literal_int => CallTarget::Foreign(parse_int(target.as_str()).unwrap()),
                            _                 => unreachable!(),
                        }
                    },
                    args: match pairs.next() {
                        Some(pair) => pair.into_inner().map(|pair| pair.try_into()).collect::<Result<Vec<_>, _>>()?,
                        None => Vec::new(),
                    },
                    thread: is_threaded,
                }
            },
            _ => bail_at!(pair.as_span(), "expected call"),
        })
    }
}

#[derive(Debug)]
pub enum Identifier {
    Named(String),
    ArrayAccess(String, Box<Expression>),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Identifier {
    type Error = Error;
    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Error> {
        Ok(match pair.as_rule() {
            Rule::id => {
                let name = pair.as_str();

                match name {
                    "if" | "else" | "switch" | "thread" | "asm" | "fun" |
                    "int" | "float" | "any" | "bool" | "null" | "uint"
                    => bail_at!(pair.as_span(), "'{}' is a reserved word", name),

                    _ => Identifier::Named(name.to_string()),
                }
            },
            Rule::arr_access => {
                let mut pairs = pair.into_inner();
                Identifier::ArrayAccess(pairs.next().unwrap().as_str().to_string(), Box::new(pairs.next().unwrap().try_into()?))
            },
            _ => bail_at!(pair.as_span(), "expected identifier"),
        })
    }
}
/*
#[derive(Debug, PartialEq, Eq)]
pub struct Variable {
    pub offset: Option<u32>,
    pub ty:     Type,
}
*/
*/
