use std::fmt;
use std::mem::transmute;
use num_enum::{TryFromPrimitive, IntoPrimitive};
use super::{Type, Signature, known_constants};

const INDENT_STRING: &str = "    ";

#[derive(Debug, Clone)]
pub struct Bytecode {
    pub offset: u32,
    pub data: Vec<(Operation, Vec<Symbol>)>,
}

#[derive(Debug)]
pub enum DataType {
    Unknown,
    Function,
    Asm,
}

impl Bytecode {
    /// Scans for pointed-to mapdata within this bytecode, attempting to infer
    /// types along the way.
    pub fn scan_data_offsets(&self) -> Vec<(DataType, u32)> {
        use Operation::*;
        use DataType::*;

        let mut ptrs = Vec::new();

        macro_rules! found {
            ($ty:expr, $sym:expr) => {
                match $sym.as_offset() {
                    Some(offset) => ptrs.push(($ty, offset)),
                    None => (),
                }
            }
        }

        for (op, args) in &self.data {
            match op {
                SetInt | SetRef => found!(Unknown, args[1]),
                Call | Exec | ExecWait | ExecRet => {
                    if let Operation::Call = op {
                        found!(Asm, args[0]);
                    } else {
                        found!(Function, args[0]);
                    }

                    match args[0] {
                        Symbol::Pointer(_) => {
                            for arg in args[1..].iter() {
                                found!(Unknown, arg)
                            }
                        },
                        Symbol::Primitive(signed) => {
                            let unsigned: u32 = unsafe { transmute(signed) };
                            if let Some((_, sig)) = known_constants::METHODS.get(&unsigned) {
                                for (n, arg) in args[1..].iter().enumerate() {
                                    // Use known signature to infer arg ptr types
                                    match sig.args[n] {
                                        Type::Function(_) => found!(Function, arg),
                                        Type::Asm(_) => found!(Asm, arg),
                                        _ => (),
                                    }
                                }
                            } else {
                                // Unknown target signature
                                for arg in args[1..].iter() {
                                    found!(Unknown, arg)
                                }
                            }
                        },
                        _ => panic!("Malformed call target: {:?}", args[0]),
                    }
                },
                End => break,
                _ => (),
            }
        }

        ptrs.dedup_by_key(|(_, offset)| offset.clone());
        ptrs
    }
}

impl fmt::Display for Bytecode {
    /// Displays as script sourcecode, which can then be parsed. **Will panic**
    /// if it discovers something unexpected (such as malformed bytecode).
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Operation::*;

        writeln!(f, "fun dump_{:x}() {{", self.offset)?;

        let mut indent_level = 1;
        macro_rules! indent {
            () => (INDENT_STRING.repeat(indent_level));
        }

        /*
        writeln!(f, "{}// temp: declaring all vars maybe used in dumped function", indent!())?;
        for i in 0..16 {
            writeln!(f, "{}any var_{:x}", indent!(), i)?;
        }
        */

        let mut is_case_state = false;
        macro_rules! is_case {
            ($e:expr) => {
                if !$e && is_case_state {
                    writeln!(f, " {{")?;
                    indent_level += 1;
                } else if $e && !is_case_state {
                    indent_level -= 1;
                    writeln!(f, "{}}}", indent!())?;
                }
                is_case_state = $e;
            }
        }

        fn write_call_exec(f: &mut fmt::Formatter, args: &mut Iterator<Item = &Symbol>) -> fmt::Result {
            let target = args.next().unwrap();
            match target {
                Symbol::Primitive(addr) => match unsafe { transmute::<i32, u32>(*addr) } {
                    0x80285960 => writeln!(f, "enter_walk(var_0)")?,

                    _ => {
                        // If we know this method, refer to it by name instead.
                        let mut sig = None;
                        match known_constants::METHODS.get(&(*addr as u32)) {
                            Some((name, s)) => {
                                sig = Some(s);
                                write!(f, "{}", name)?;
                            },
                            None => write!(f, "0x{:X}", addr)?,
                        }

                        // Argument list
                        writeln!(f, "({})", args
                            .enumerate()
                            .map(|(n, arg)| {
                                // Type hints, if we know the signature.
                                if let Some(Signature { args, .. }) = sig {
                                    if n >= args.len() {
                                        panic!("Too many arguments in call: 0x{:X}", addr);
                                    }

                                    let ty = &args[n];
                                    if let super::Type::Bool = ty {
                                        format!("{}", arg.as_bool()
                                            .expect(&format!("Expected bool arg {} in call: 0x{:X}", n, addr)))
                                    } else {
                                        format!("{}", arg)
                                    }
                                } else {
                                    // Print args as hex, since we don't
                                    // know what type they actually are.
                                    format!("{:X}", arg)
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(", ")
                        )?;
                    }
                },
                _ => {
                    writeln!(f, "{:X}({})", target, args
                        .map(|arg| {
                            // Print args as hex, since we don't yet know
                            // what type they actually are.
                            format!("{:X}", arg)
                        })
                        .collect::<Vec<String>>()
                        .join(", "))?;
                },
            }

            Ok(())
        }

        for (op, args) in &self.data {
            match op {
                End => break,
                Return => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);
                    writeln!(f, "{}return", indent!())?;
                },

                Label => {
                    assert_eq!(args.len(), 1);
                    is_case!(false);
                    writeln!(f, "{}label .o{}", indent!(), args[0])?;
                },
                Goto => {
                    assert_eq!(args.len(), 1);
                    is_case!(false);
                    writeln!(f, "{}goto .o{}", indent!(), args[0])?;
                },

                Loop => {
                    assert_eq!(args.len(), 1);
                    is_case!(false);
                    if args[0].as_u32().or(Some(1)).unwrap() == 0 {
                        // Infinite loop
                        writeln!(f, "{}loop {{", indent!())?;
                    } else {
                        writeln!(f, "{}loop {} {{", indent!(), args[0])?;
                    }
                    indent_level += 1;
                },
                EndLoop => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);
                    indent_level -= 1;
                    writeln!(f, "{}}}", indent!())?;
                },
                BreakLoop => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);
                    writeln!(f, "{}break loop", indent!())?;
                },

                Wait => {
                    assert_eq!(args.len(), 1);
                    is_case!(false);
                    writeln!(f, "{}sleep {}", indent!(), args[0])?;
                },
                WaitSeconds => {
                    assert_eq!(args.len(), 1);
                    is_case!(false);
                    writeln!(f, "{}sleep {} secs", indent!(), args[0])?;
                },

                IfEq => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}if {} == {} {{", indent!(), args[0], args[1])?;
                    indent_level += 1;
                },
                IfNe => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}if {} != {} {{", indent!(), args[0], args[1])?;
                    indent_level += 1;
                },
                IfLt => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}if {} < {} {{", indent!(), args[0], args[1])?;
                    indent_level += 1;
                },
                IfGt => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}if {} > {} {{", indent!(), args[0], args[1])?;
                    indent_level += 1;
                },
                IfLte => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}if {} <= {} {{", indent!(), args[0], args[1])?;
                    indent_level += 1;
                },
                IfGte => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}if {} >= {} {{", indent!(), args[0], args[1])?;
                    indent_level += 1;
                },
                IfAndNz => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}if {:b} !& {:b} {{", indent!(), args[0], args[1])?;
                    indent_level += 1;
                },
                IfAndZ => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}if {:b} & {:b} {{", indent!(), args[0], args[1])?;
                    indent_level += 1;
                },
                Else => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);
                    indent_level -= 1;
                    writeln!(f, "{}}} else {{", indent!())?;
                    indent_level += 1;
                },
                EndIf => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);
                    indent_level -= 1;
                    writeln!(f, "{}}}", indent!())?;
                },

                Switch | SwitchConst => {
                    // note: SwitchConst goes *completely unused* in vanilla, so
                    //       if we do find it we pretend its a normal Switch.

                    assert_eq!(args.len(), 1);

                    if is_case_state == true {
                        panic!("Found illegal double switch operation");
                    }
                    is_case_state = true;

                    writeln!(f, "{}switch {} {{", indent!(), args[0])?;
                    indent_level += 1;
                },
                CaseEq => {
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case == {}", indent!(), args[0])?;
                },
                CaseNe => {
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case != {}", indent!(), args[0])?;
                },
                CaseLt => {
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case < {}", indent!(), args[0])?;
                },
                CaseGt => {
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case > {}", indent!(), args[0])?;
                },
                CaseLte => {
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case <= {}", indent!(), args[0])?;
                },
                CaseGte => {
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case > {}", indent!(), args[0])?;
                },
                CaseDefault => {
                    assert_eq!(args.len(), 0);
                    is_case!(true);
                    write!(f, "{}default", indent!())?;
                },
                CaseOrEq => {
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case or == {}", indent!(), args[0])?;
                },
                CaseAndEq => {
                    // note: unused in vanilla, and doesn't make much sense
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case and == {}", indent!(), args[0])?;
                },
                CaseAndZ => {
                    assert_eq!(args.len(), 1);
                    is_case!(true);
                    write!(f, "{}case & {}", indent!(), args[0])?;
                },
                EndCaseGroup => {
                    assert_eq!(args.len(), 0);
                    if is_case_state == true {
                        // Empty block.
                        writeln!(f, "{{}}")?;
                        is_case_state = false;
                    }
                },
                CaseRange => {
                    assert_eq!(args.len(), 2);
                    is_case!(true);
                    write!(f, "{}case {}..{}", indent!(), args[0], args[1])?;
                },
                BreakCase => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);
                    writeln!(f, "{}break case", indent!())?;
                },
                EndSwitch => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);

                    indent_level -= 1;
                    writeln!(f, "{}}}", indent!())?;

                    indent_level -= 1;
                    writeln!(f, "{}}}", indent!())?;
                },

                SetInt | SetFloat => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);

                    if args[0].is_flag() {
                        // Boolean.
                        writeln!(f, "{}{} = {}", indent!(), args[0], args[1].as_bool().expect("Flag set to non-bool value"))?;
                    } else {
                        // Numeric.
                        writeln!(f, "{}{} = {}", indent!(), args[0], args[1])?;
                    }
                },
                SetRef => panic!("SetRef op unimplemented"), // TODO

                AddInt | AddFloat => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}{} += {}", indent!(), args[0], args[1])?;
                },
                SubInt | SubFloat => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}{} -= {}", indent!(), args[0], args[1])?;
                },
                MulInt | MulFloat => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}{} *= {}", indent!(), args[0], args[1])?;
                },
                DivInt | DivFloat => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}{} /= {}", indent!(), args[0], args[1])?;
                },
                ModInt => {
                    assert_eq!(args.len(), 2);
                    is_case!(false);
                    writeln!(f, "{}{} %= {}", indent!(), args[0], args[1])?;
                },

                // TODO
                UseIntBuffer | Get1Int | Get2Int | Get3Int | Get4Int | GetIntN =>
                    panic!("IntBuffer ops unimplemented"),

                // TODO
                UseFloatBuffer | Get1Float | Get2Float | Get3Float | Get4Float | GetFloatN =>
                    panic!("FloatBuffer ops unimplemented"),

                UseArray | UseFlagArray => {
                    assert_eq!(args.len(), 1);
                    is_case!(false);
                    writeln!(f, "{}arr = {:X}", indent!(), args[0])?;
                },
                AllocArray => panic!("AllocArray op unimplemented"), // TODO

                // TODO
                And | AndRef | Or | OrRef => panic!("Bitwise ops unimplemented"),

                Call | ExecWait => {
                    is_case!(false);
                    write!(f, "{}", indent!())?;
                    write_call_exec(f, &mut args.iter())?;
                },
                Exec => {
                    is_case!(false);

                    write!(f, "{}thread ", indent!())?;
                    write_call_exec(f, &mut args.iter())?;
                },
                ExecRet => {
                    is_case!(false);

                    let var = &args[1];

                    // Create the arglist iterator, skipping args[1] (since that
                    // is `var`).
                    let mut it = args
                        .into_iter()
                        .enumerate()
                        .filter_map(|(i, e)| if i != 2 { Some(e) } else { None });

                    write!(f, "{}{} = thread ", indent!(), var)?;
                    write_call_exec(f, &mut it)?;
                },

                Bind => {
                    assert_eq!(args.len(), 5);
                    is_case!(false);

                    if let Symbol::Primitive(n) = args[4] {
                        assert_eq!(n, 0);
                        writeln!(f, "{}bind_{}({:X}, {})", indent!(), args[1].hint_trigger(), args[0], args[2])?;
                    } else {
                        // TODO check this is actually how it works
                        writeln!(f, "{}{:X} = bind_{}({:X}, {})", indent!(), args[4], args[1].hint_trigger(), args[0], args[2])?;
                    }
                },
                BindLock => panic!("BindLock op unimplemented"), // TODO
                Unbind => panic!("Unbind op unimplemented"), // TODO

                Kill => {
                    assert_eq!(args.len(), 1);
                    is_case!(false);
                    writeln!(f, "{}kill {}", indent!(), args[0])?;
                },
                Jump => panic!("Jump op unimplemented"), // TODO

                // TODO
                SetPriority | SetTimescale | SetSuspensionGroup |
                SuspendAll | ResumeAll | SuspendOthers | ResumeOthers | Suspend | Resume |
                DoesScriptExist => panic!("Runtime modifier ops unimplemented"),

                Thread => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);
                    writeln!(f, "{}thread {{", indent!())?;
                    indent_level += 1;
                },
                ChildThread => panic!("ChildThread op unimplemented"), // TODO
                EndThread | EndChildThread => {
                    assert_eq!(args.len(), 0);
                    is_case!(false);
                    indent_level -= 1;
                    writeln!(f, "{}}}", indent!())?;
                },
            }
        }

        if indent_level != 1 {
            panic!("Control flow level mismatch");
        }

        write!(f, "}}")?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Unknown(i32),
    Float(f32),
    FlagArray(i32),
    Array(i32),
    GameByte(u32),
    AreaByte(i32),
    GameFlag(i32),
    AreaFlag(i32),
    MapFlag(i32),
    Flag(i32),
    MapVar(i32),
    Var(i32),
    Primitive(i32),
    Pointer(u32),
    Trigger(Trigger),
}

#[derive(Debug, PartialEq, Eq, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum Trigger {
    FloorTouch    = 0x00000080,
    FloorAbove    = 0x00080000,
    FloorInteract = 0x00000800,
    FloorJump     = 0x00000200,

    WallTouch     = 0x00000400,
    WallPush      = 0x00000040, // ?
    WallInteract  = 0x00000100,
    WallHammer    = 0x00001000,

    Bomb          = 0x00100000,
    GameFlagSet   = 0x00010000,
    AreaFlagSet   = 0x00020000,
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Trigger::*;

        match &self {
            FloorTouch    => write!(f, "stand_on"),
            FloorAbove    => write!(f, "above_floor"),
            FloorInteract => write!(f, "interact_floor"),
            FloorJump     => write!(f, "jump"),

            WallTouch     => write!(f, "touch_wall"),
            WallPush      => write!(f, "push_wall"),
            WallInteract  => write!(f, "interact_wall"),
            WallHammer    => write!(f, "hammer_wall"),

            Bomb          => write!(f, "bomb"),
            GameFlagSet   => write!(f, "gameflag_change"),
            AreaFlagSet   => write!(f, "areaflag_change"),
        }
    }
}

impl Symbol {
    fn hint_trigger(&self) -> Symbol {
        use std::convert::TryFrom;

        match &self {
            Symbol::Primitive(value) => match Trigger::try_from(*value as u32) {
                Ok(trigger) => Symbol::Trigger(trigger),
                Err(_) => self.clone(),
            },
            _ => self.clone(),
        }
    }

    fn is_flag(&self) -> bool {
        use Symbol::*;

        match &self {
            GameFlag(_) | AreaFlag(_) | MapFlag(_) | Flag(_) => true,
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        use Symbol::*;

        match &self {
            Primitive(v) => Some(*v == 1),
            _ => None,
        }
    }

    fn as_offset(&self) -> Option<u32> {
        use Symbol::*;

        match &self {
            Pointer(offset) => Some(*offset),
            _ => None,
        }
    }

    fn as_u32(&self) -> Option<u32> {
        use Symbol::*;

        match &self {
            Primitive(val) => Some(unsafe { transmute(*val) }),
            _ => None,
        }
    }
}

impl From<u32> for Symbol {
    fn from(u: u32) -> Symbol {
        let s: i32 = unsafe { transmute(u) };

        if s <= -270000000 {
            if u & 0xFFFF0000 == 0x80240000 {
                Symbol::Pointer(u.wrapping_sub(0x80240000))
            } else {
                Symbol::Primitive(s)
            }
        } else if s <= -250000000 {
            Symbol::Unknown(s + 250000000)
        } else if s <= -220000000 {
            Symbol::Float(((s + 230000000) as f32) / 1024.0)
        } else if s <= -200000000 {
            Symbol::FlagArray(s + 210000000)
        } else if s <= -180000000 {
            Symbol::Array(s + 190000000)
        } else if s <= -160000000 {
            Symbol::GameByte((s + 170000000) as u32)
        } else if s <= -140000000 {
            Symbol::AreaByte(s + 150000000)
        } else if s <= -120000000 {
            Symbol::GameFlag(s + 130000000)
        } else if s <= -100000000 {
            Symbol::AreaFlag(s + 110000000)
        } else if s <= -80000000 {
            Symbol::MapFlag(s + 90000000)
        } else if s <= -60000000 {
            Symbol::Flag(s + 70000000)
        } else if s <= -40000000 {
            Symbol::MapVar(s + 50000000)
        } else if s <= -20000000 {
            Symbol::Var(s + 30000000)
        } else {
            Symbol::Primitive(s)
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Symbol::*;

        // TODO: other `known_constants`

        match &self {
            Unknown(ptr)     => write!(f, "unknown_{:x}", ptr),
            Float(value)     => write!(f, "{}", value),
            FlagArray(index) => write!(f, "arr_{}", index),
            Array(index)     => write!(f, "arr_{}", index),
            GameByte(index) => match known_constants::GAMEBYTES.get(index) {
                Some(name) => write!(f, "{}", name),
                None       => write!(f, "global_{}", index),
            },
            AreaByte(index)  => write!(f, "area_{}", index),
            GameFlag(index)  => write!(f, "global_flag_{}", index),
            AreaFlag(index)  => write!(f, "area_flag_{}", index),
            MapFlag(index)   => write!(f, "map_flag_{}", index),
            Flag(index)      => write!(f, "bool_{:x}", index),
            MapVar(index)    => write!(f, "map_{:x}", index),
            Var(index)       => write!(f, "var_{:x}", index),
            Primitive(value) => write!(f, "{}", value),
            Pointer(offset)  => write!(f, "dump_{:x}", offset),
            Trigger(trigger) => write!(f, "{}", trigger),
        }
    }
}

impl fmt::UpperHex for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Symbol::*;

        match &self {
            Primitive(value) => write!(f, "0x{:08X}", value),
            _                => write!(f, "{}", self),
        }
    }
}

impl fmt::Binary for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Symbol::*;

        match &self {
            Primitive(value) => write!(f, "0b{:b}", value),
            _                => write!(f, "{}", self),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Operation {
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
    Kill, Jump, // Jump???
    SetPriority, SetTimescale, SetSuspensionGroup,
    BindLock,
    SuspendAll, ResumeAll, SuspendOthers, ResumeOthers, Suspend, Resume,
    DoesScriptExist, Thread, EndThread, ChildThread, EndChildThread,
}
