use std::convert::TryInto;
use std::collections::{VecDeque, HashSet};
use std::cell::RefCell;
use failure_derive::*;
use num_enum::{TryFromPrimitive, IntoPrimitive};
use super::datatype::DataType;
use super::parse::ast::*;
use super::globals::*;
use super::Scope;

#[derive(Debug, Clone)]
pub struct Bytecode {
    data: VecDeque<Operation>,
    seen_identifiers: HashSet<Arg>,
}

pub type Operation = (Opcode, Vec<Arg>);

impl Bytecode {
    pub fn read(rom: &mut crate::rom::Rom) -> Bytecode {
        let mut data = VecDeque::new();
        loop {
            let opcode = rom.read_u32();

            let mut args = Vec::new();
            for _ in 0..rom.read_u32() {
                args.push(Arg(rom.read_u32()))
            }

            match opcode.try_into().ok() {
                Some(op) => {
                    data.push_back((op, args));

                    if let Opcode::End = op {
                        return Bytecode {
                            data,
                            seen_identifiers: HashSet::new(),
                        };
                    }
                },
                None => panic!("unknown opcode: {:02X}", opcode),
            }
        }
    }

    pub fn decompile(mut self, scope: &mut Scope) -> Result<Vec<Statement>, Error> {
        let mut stmts = Vec::new();
        loop {
            // See if we've reached the end
            if let (Opcode::End, _) = self.peek_op()? {
                return if self.data.len() > 1 {
                    // There's still data left after the End opcode :/
                    Err(Error::UnexpectedEnd)
                } else {
                    // Remove pointless trailing return statement, if there is one.
                    if let Statement::Return = stmts[stmts.len() - 1] {
                        stmts.pop();
                    }

                    // Bring local variables (FunWords and FunFlags) into scope.
                    for oparg in self.seen_identifiers.into_iter() {
                        match oparg.kind() {
                            ArgKind::FunWord => {
                                let name = oparg.into_identifier().unwrap().0;
                                scope.insert_name(name, DataType::Any);
                            },
                            ArgKind::FunFlag => {
                                let name = oparg.into_identifier().unwrap().0;
                                scope.insert_name(name, DataType::Bool);
                            },
                            _ => (),
                        }
                    }

                    Ok(stmts)
                };
            }

            // Decode and consume the next statement
            stmts.append(&mut self.decompile_op()?);
        }
    }

    fn consume_op(&mut self) -> Result<Operation, Error> {
        self.data.pop_front().ok_or(Error::MissingEnd)
    }

    fn peek_op(&self) -> Result<&Operation, Error> {
        self.data.get(0).ok_or(Error::MissingEnd)
    }

    fn decompile_op(&mut self) -> Result<Vec<Statement>, Error> {
        let (opcode, opargs) = self.consume_op()?;
        match opcode {
            Opcode::IfEq  | Opcode::IfNe  | Opcode::IfLt    | Opcode::IfGt |
            Opcode::IfLte | Opcode::IfGte | Opcode::IfAndNz | Opcode::IfAndZ
            => Ok(vec![Statement::If {
                condition: Expression::Operation {
                    lhs: Box::new(opargs.get(0)
                        .ok_or_else(|| Error::MissingArg(opcode, 0))?
                        .into_expression()),
                    op:  opcode.into_operator().unwrap(),
                    rhs: Box::new(opargs.get(1)
                        .ok_or_else(|| Error::MissingArg(opcode, 1))?
                        .into_expression()),
                },
                block_true: {
                    let mut stmts = Vec::new();
                    loop {
                        match self.peek_op()? {
                            // Consume Else; block_false does NOT expect it
                            (Opcode::Else, _) => {
                                self.consume_op()?;
                                break
                            },

                            // Don't consume EndIf; block_false needs it
                            (Opcode::EndIf, _) => break,

                            // Consume it!! >:D
                            _ => stmts.append(&mut self.decompile_op()?),
                        };
                    }
                    stmts
                },
                block_false: {
                    let mut stmts = Vec::new();
                    loop {
                        match self.peek_op()? {
                            (Opcode::EndIf, _) => {
                                self.consume_op()?;
                                break
                            },
                            _ => stmts.append(&mut self.decompile_op()?),
                        };
                    }
                    stmts
                },
            }]),

            Opcode::Switch | Opcode::SwitchConst => Ok(vec![Statement::Switch {
                expression: opargs.get(0)
                    .ok_or_else(|| Error::MissingArg(opcode, 0))?
                    .into_expression(),
                cases: {
                    let mut cases = Vec::new();
                    loop {
                        match self.peek_op()? {
                            // Consume Case ops:
                            (Opcode::CaseEq, _)      |
                            (Opcode::CaseOrEq, _)    |
                            (Opcode::CaseNe, _)      |
                            (Opcode::CaseLt, _)      |
                            (Opcode::CaseGt, _)      |
                            (Opcode::CaseLte, _)     |
                            (Opcode::CaseGte, _)     |
                            (Opcode::CaseAndZ, _)    |
                            (Opcode::CaseDefault, _) => {
                                let (case_opcode, case_opargs) = self.consume_op()?;

                                let mut stmts = Vec::new();
                                loop {
                                    match self.peek_op()? {
                                        (Opcode::CaseEq, _)      |
                                        (Opcode::CaseAndEq, _)   |
                                        (Opcode::CaseOrEq, _)    |
                                        (Opcode::CaseNe, _)      |
                                        (Opcode::CaseLt, _)      |
                                        (Opcode::CaseGt, _)      |
                                        (Opcode::CaseLte, _)     |
                                        (Opcode::CaseGte, _)     |
                                        (Opcode::CaseAndZ, _)    |
                                        (Opcode::CaseDefault, _) |
                                        (Opcode::EndSwitch, _)   => break,

                                        _ => stmts.append(&mut self.decompile_op()?),
                                    };
                                }

                                cases.push((if let Opcode::CaseDefault = case_opcode {
                                    Case::Default
                                } else {
                                    Case::Test {
                                        operator: case_opcode.into_operator().unwrap(),
                                        against: case_opargs.get(0)
                                            .ok_or_else(|| Error::MissingArg(case_opcode, 0))?
                                            .into_expression(),
                                    }
                                }, stmts));
                            },

                            // This goes unused in vanilla, and is... useless.
                            (Opcode::CaseAndEq, _) =>
                                return Err(Error::UnimplementedOpcode(Opcode::CaseAndEq)),

                            // Close the switch, consuming the EndSwitch op.
                            (Opcode::EndSwitch, _) => {
                                self.consume_op()?;
                                break
                            },

                            // A couple vanilla functions have weird, malformed
                            // switches that are not followed by any cases (and
                            // lack an EndSwitch). Thus, we just exit the switch
                            // if we see an unexpected opcode inside it.
                            _ => break,
                        };
                    }
                    cases
                },
            }]),

            Opcode::SetInt | Opcode::SetRef | Opcode::SetFloat => {
                let identifier_arg = opargs.get(0)
                    .ok_or_else(|| Error::MissingArg(opcode, 0))?;
                let identifier = identifier_arg.into_identifier()
                    .ok_or_else(|| Error::BadArg(opcode, 0))?;

                let expression = RefCell::new(opargs.get(1)
                    .ok_or_else(|| Error::MissingArg(opcode, 1))?
                    .into_expression());

                // If we haven't seen the identifier yet, declare it.
                if !self.seen_identifiers.contains(&identifier_arg) {
                    self.seen_identifiers.insert(*identifier_arg);

                    // Only declare identifiers that this function owns.
                    match identifier_arg.kind() {
                        ArgKind::FunWord => return Ok(vec![Statement::VarDeclare {
                            datatype: RefCell::new(match opcode {
                                // Floats are *always* floats, but bytecode ints
                                // are sometimes pointers or some other datatype.
                                // We'll leave detecting that to type inference.
                                Opcode::SetFloat => DataType::Float,
                                _                => DataType::Any,
                            }),
                            identifier,
                            expression: Some(expression),
                        }]),

                        ArgKind::FunFlag => return Ok(vec![Statement::VarDeclare {
                            datatype: RefCell::new(DataType::Bool),
                            identifier,
                            expression: Some(expression),
                        }]),

                        _ => (),
                    };
                }

                // If we've reached here, it's just assignment; no declaration needed.
                Ok(vec![Statement::VarAssign { identifier, expression }])
            },

            Opcode::Call | Opcode::ExecWait | Opcode::Exec => Ok(vec![Statement::MethodCall {
                method: opargs.get(0)
                    .ok_or_else(|| Error::MissingArg(opcode, 0))?
                    .into_ident_or_ptr()
                    .ok_or_else(|| Error::BadArg(opcode, 0))?,
                arguments: opargs.iter()
                    .skip(1)
                    .map(|oparg| RefCell::new(oparg.into_expression()))
                    .collect(),
                threading: match opcode {
                    Opcode::Exec => MethodThreading::Yes,
                    _            => MethodThreading::No,
                },
            }]),
            Opcode::ExecRet => Ok(vec![Statement::MethodCall {
                method: opargs.get(0)
                    .ok_or_else(|| Error::MissingArg(opcode, 0))?
                    .into_ident_or_ptr()
                    .ok_or_else(|| Error::BadArg(opcode, 0))?,
                arguments: opargs.iter()
                    .skip(2)
                    .map(|oparg| RefCell::new(oparg.into_expression()))
                    .collect(),
                threading: MethodThreading::Assign(opargs.get(1)
                    .ok_or_else(|| Error::MissingArg(opcode, 1))?
                    .into_identifier()
                    .ok_or_else(|| Error::BadArg(opcode, 1))?),
            }]),

            Opcode::Wait | Opcode::WaitSeconds => Ok(vec![Statement::Wait {
                time: opargs.get(0)
                    .ok_or_else(|| Error::MissingArg(opcode, 0))?
                    .into_expression(),
                unit: match opcode {
                    Opcode::Wait => TimeUnit::Frames,
                    Opcode::WaitSeconds => TimeUnit::Seconds,
                    _ => unreachable!(),
                },
            }]),

            Opcode::Label => Ok(vec![Statement::Label {
                name: opargs.get(0)
                    .ok_or_else(|| Error::MissingArg(opcode, 0))?
                    .into_int()
                    .ok_or_else(|| Error::BadArg(opcode, 0))?
                    .to_string(),
            }]),
            Opcode::Goto => Ok(vec![Statement::Goto {
                label_name: opargs.get(0)
                    .ok_or_else(|| Error::MissingArg(opcode, 0))?
                    .into_int()
                    .ok_or_else(|| Error::BadArg(opcode, 0))?
                    .to_string(),
            }]),

            Opcode::Return => Ok(vec![Statement::Return]),

            Opcode::End => Err(Error::UnexpectedEnd),
            _           => Err(Error::UnimplementedOpcode(opcode)),
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "opcode {:?} is unimplemented", _0)]
    UnimplementedOpcode(Opcode),

    #[fail(display = "missing End opcode")]
    MissingEnd,

    #[fail(display = "unexpected End opcode")]
    UnexpectedEnd,

    #[fail(display = "opcode {:?} is missing arg{}", _0, _1)]
    MissingArg(Opcode, u8),

    #[fail(display = "opcode {:?} has bad arg{}", _0, _1)]
    BadArg(Opcode, u8),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)] // The game stores opcodes as words, so we will too.
pub enum Opcode {
    End = 1, Return,
    Label, Goto,
    Loop, EndLoop, BreakLoop,
    Wait, WaitSeconds,
    IfEq, IfNe, IfLt, IfGt, IfLte, IfGte, IfAndNz, IfAndZ, Else, EndIf,
    Switch, SwitchConst, CaseEq, CaseNe, CaseLt, CaseGt, CaseLte, CaseGte, CaseDefault,
    CaseOrEq, CaseAndEq, CaseAndZ, EndCaseGroup, CaseRange, BreakCase, EndSwitch,
    SetInt, SetRef, SetFloat,
    AddInt, SubInt, MulInt, DivInt, ModInt,
    AddFloat, SubFloat, MulFloat, DivFloat,
    UseIntBuffer, Get1Int, Get2Int, Get3Int, Get4Int, GetIntN,
    UseFloatBuffer, Get1Float, Get2Float, Get3Float, Get4Float, GetFloatN,
    UseArray, UseFlagArray, AllocArray,
    And, AndRef, Or, OrRef, // Unused?
    Call, Exec, ExecRet, ExecWait,
    Bind, Unbind,
    Kill, Jump,
    SetPriority, SetTimescale, SetSuspensionGroup,
    BindLock,
    SuspendAll, ResumeAll, SuspendOthers, ResumeOthers, Suspend, Resume,
    DoesScriptExist, Thread, EndThread, ChildThread, EndChildThread,
}

impl Opcode {
    pub fn into_operator(self) -> Option<Operator> {
        match self {
            Opcode::IfEq    => Some(Operator::Eq),
            Opcode::IfNe    => Some(Operator::Ne),
            Opcode::IfLt    => Some(Operator::Lt),
            Opcode::IfGt    => Some(Operator::Gt),
            Opcode::IfLte   => Some(Operator::Lte),
            Opcode::IfGte   => Some(Operator::Gte),
            Opcode::IfAndNz => Some(Operator::BitAndNz),
            Opcode::IfAndZ  => Some(Operator::BitAndZ),

            Opcode::CaseEq      => Some(Operator::Eq),
            Opcode::CaseOrEq    => Some(Operator::Eq),
            Opcode::CaseAndEq   => Some(Operator::Eq),
            Opcode::CaseNe      => Some(Operator::Ne),
            Opcode::CaseLt      => Some(Operator::Lt),
            Opcode::CaseGt      => Some(Operator::Gt),
            Opcode::CaseLte     => Some(Operator::Lte),
            Opcode::CaseGte     => Some(Operator::Gte),
            Opcode::CaseAndZ    => Some(Operator::BitAndZ),

            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Arg(u32);

#[derive(Debug)]
pub enum ArgKind {
    Int, Float,
    GameByte, AreaByte, MapWord, FunWord,
    GameFlag, AreaFlag, MapFlag, FunFlag,
    FlagArrayIndex, ArrayIndex,
}

impl Arg {
    pub fn into_expression(self) -> Expression {
        let s = self.as_signed();

        match self.kind() {
            ArgKind::Int   => Expression::LiteralInt(self.0),
            ArgKind::Float => Expression::LiteralFloat(((s + 230000000) as f32) / 1024.0),

            ArgKind::FlagArrayIndex =>
                Expression::ArrayIndex(Identifier(FLAGARRAY_STR.to_string()), (s + 190000000) as u8),
            ArgKind::ArrayIndex =>
                Expression::ArrayIndex(Identifier(ARRAY_STR.to_string()), (s + 210000000) as u8),

            _ => Expression::Identifier(self.into_identifier().unwrap()),
        }
    }

    pub fn into_identifier(self) -> Option<Identifier> {
        let s = self.as_signed();

        // TODO: check all the calculations here are correct
        match self.kind() {
            ArgKind::GameByte =>
                Some(Identifier(format!("{}_{:X}", GAMEBYTE_STR, (s + 170000000) as u32))),
            ArgKind::AreaByte =>
                Some(Identifier(format!("{}_{:X}", AREABYTE_STR, s + 150000000))),
            ArgKind::MapWord =>
                Some(Identifier(format!("{}_{:X}", MAPWORD_STR, s + 50000000))),
            ArgKind::FunWord =>
                Some(Identifier(format!("{}_{:X}", FUNWORD_STR, s + 30000000))),

            ArgKind::GameFlag =>
                Some(Identifier(format!("{}_{:X}", GAMEFLAG_STR, s + 130000000))),
            ArgKind::AreaFlag =>
                Some(Identifier(format!("{}_{:X}", AREAFLAG_STR, s + 110000000))),
            ArgKind::MapFlag =>
                Some(Identifier(format!("{}_{:X}", MAPFLAG_STR, s + 90000000))),
            ArgKind::FunFlag =>
                Some(Identifier(format!("{}_{:X}", FUNFLAG_STR, s + 70000000))),

            _ => None,
        }
    }

    pub fn into_ident_or_ptr(self) -> Option<IdentifierOrPointer> {
        match self.kind() {
            ArgKind::Int => Some(IdentifierOrPointer::Pointer(self.0)),
            _ => None,
        }
    }

    pub fn into_int(self) -> Option<i32> {
        match self.kind() {
            ArgKind::Int => Some(self.0 as i32),
            _ => None,
        }
    }

    fn as_signed(self) -> i32 {
        self.0 as i32
    }

    #[allow(clippy::if_same_then_else)]
    pub fn kind(self) -> ArgKind {
        let s = self.as_signed();

        if s <= -270000000 {
            ArgKind::Int
        } else if s <= -250000000 {
            // Clover labeled this 'Unknown' - could be worth looking into...?
            ArgKind::Int
        } else if s <= -220000000 {
            ArgKind::Float
        } else if s <= -200000000 {
            ArgKind::FlagArrayIndex
        } else if s <= -180000000 {
            ArgKind::ArrayIndex
        } else if s <= -160000000 {
            ArgKind::GameByte
        } else if s <= -140000000 {
            ArgKind::AreaByte
        } else if s <= -120000000 {
            ArgKind::GameFlag
        } else if s <= -100000000 {
            ArgKind::AreaFlag
        } else if s <= -80000000 {
            ArgKind::MapFlag
        } else if s <= -60000000 {
            ArgKind::FunFlag
        } else if s <= -40000000 {
            ArgKind::MapWord
        } else if s <= -20000000 {
            ArgKind::FunWord
        } else {
            ArgKind::Int
        }
    }
}
