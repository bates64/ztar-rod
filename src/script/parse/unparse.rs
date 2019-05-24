use std::cell::RefCell;
use itertools::Itertools;
use super::ast::*;
use super::super::Scope;

/// Trait for structs that can produce a script sourcecode equivalent of
/// themselves, given a Scope to look-up pointers.
pub trait Unparse {
    fn unparse(self, scope: &Scope) -> String;
}

/// Adds indentation to each line of the given String.
fn indent(string: String) -> String {
    string
        .lines()
        .map(|line| format!("    {}", line)) // 4 spaces
        .join("\n")
}

impl Unparse for Declaration {
    fn unparse(self, scope: &Scope) -> String {
        match self {
            Declaration::Fun { name, arguments, block } =>
                format!("fun {}({}) {{\n{}\n}}",
                    name.unparse(scope),
                    arguments
                        .into_iter()
                        .map(|(id, ty)| format!("{}: {}", id.unparse(scope), ty.unparse(scope)))
                        .join(", "),
                    indent(block.unparse(scope)),
                ),
        }
    }
}

impl Unparse for Statement {
    fn unparse(self, scope: &Scope) -> String {
        match self {
            Statement::Return => "return".to_string(),

            Statement::Label { name } => format!(".{}", name),
            Statement::Goto { label_name } => format!("goto .{}", label_name),

            Statement::VarAssign { identifier, operator, expression } =>
                format!("{} {} {}",
                    identifier.unparse(scope),
                    match operator {
                        AssignmentOperator::Eq => "=",
                        AssignmentOperator::Add => "+=",
                        AssignmentOperator::Sub => "-=",
                        AssignmentOperator::Mul => "*=",
                        AssignmentOperator::Div => "/=",
                        AssignmentOperator::Mod => "%=",
                    },
                    expression.into_inner().unparse(scope),
                ),

            Statement::VarDeclare { identifier, datatype, expression } => match datatype.into_inner() {
                DataType::Any => match expression {
                    Some(expression) => format!("var {} = {}",
                        identifier.unparse(scope),
                        expression.into_inner().unparse(scope),
                    ),
                    None => format!("var {}",
                        identifier.unparse(scope),
                    ),
                },
                datatype => match expression {
                    Some(expression) => format!("var {}: {} = {}",
                        identifier.unparse(scope),
                        datatype.unparse(scope),
                        expression.into_inner().unparse(scope),
                    ),
                    None => format!("var {}: {}",
                        identifier.unparse(scope),
                        datatype.unparse(scope),
                    ),
                },
            },

            Statement::MethodCall { method, arguments, threading, .. } => match threading {
                MethodThreading::Assign(ident) => format!("{} = thread {}({})",
                    ident.unparse(scope),
                    method.unparse(scope),
                    arguments.unparse(scope),
                ),
                MethodThreading::Yes => format!("thread {}({})",
                    method.unparse(scope),
                    arguments.unparse(scope),
                ),
                MethodThreading::No => format!("{}({})",
                    method.unparse(scope),
                    arguments.unparse(scope),
                ),
            },

            Statement::Wait { time, unit } => match unit {
                TimeUnit::Frames  => format!("wait {}",     time.unparse(scope)),
                TimeUnit::Seconds => format!("waitsecs {}", time.unparse(scope)),
            },

            Statement::Bind { trigger, dispatch } => format!("bind {} {}",
                trigger.unparse(scope),
                dispatch.unparse(scope)
            ),
            Statement::Unbind => "unbind".to_string(),

            Statement::BreakLoop => "breakloop".to_string(),
            Statement::BreakCase => "breakcase".to_string(),

            Statement::If { condition, block_true, mut block_false } => match block_false.len() {
                // No else block
                0 => format!("if {} {{\n{}\n}}",
                    condition.unparse(scope),
                    indent(block_true.unparse(scope)),
                ),

                // Only one stmt in else block
                1 => match block_false[0] {
                    // 'else if' contraction
                    Statement::If { .. } => format!("if {} {{\n{}\n}} else {}",
                        condition.unparse(scope),
                        indent(block_true.unparse(scope)),
                        block_false.pop().unwrap().unparse(scope), // pop because we require ownership
                    ),

                    // Treat else block as normal
                    _ => format!("if {} {{\n{}\n}} else {{\n{}\n}}",
                        condition.unparse(scope),
                        indent(block_true.unparse(scope)),
                        indent(block_false.unparse(scope)),
                    ),
                },

                // Has else block
                _ => format!("if {} {{\n{}\n}} else {{\n{}\n}}",
                    condition.unparse(scope),
                    indent(block_true.unparse(scope)),
                    indent(block_false.unparse(scope)),
                ),
            },

            Statement::Switch { expression, cases } => format!("switch {} {{\n{}\n}}",
                expression.unparse(scope),
                indent(cases
                    .into_iter()
                    .map(|(case, block)| match case {
                        Case::Default => format!("default {{\n{}\n}}", indent(block.unparse(scope))),
                        Case::Test { operator, against } => format!("case {} {} {{\n{}\n}}",
                            operator.unparse(scope),
                            against.unparse(scope),
                            indent(block.unparse(scope)),
                        ),
                    })
                    .join("\n")
                ),
            ),

            Statement::Thread { block } => format!("thread {{\n{}\n}}",
                indent(block.unparse(scope))),

            Statement::Loop { block, times } => match times {
                LoopTimes::Infinite => format!("loop {{\n{}\n}}",
                    indent(block.unparse(scope))),

                LoopTimes::Expression(expr) => format!("loop {} {{\n{}\n}}",
                    expr.unparse(scope),
                    indent(block.unparse(scope)))
            },
        }
    }
}

impl Unparse for Expression {
    fn unparse(self, scope: &Scope) -> String {
        match self {
            Expression::LiteralInt(maybe_ptr) => {
                if maybe_ptr & 0xFFFF_0000 == 0x8024_0000 {
                    // It's probably a script pointer; try to give it its name
                    if let Some(name) = scope.lookup_ptr(maybe_ptr) {
                        return name.to_string();
                    }
                }

                // It's an int. Format it as signed.
                format!("{}", unsafe { std::mem::transmute::<u32, i32>(maybe_ptr) })
            },
            Expression::LiteralFloat(f) => format!("{:?}", f),
            Expression::LiteralBool(b)  => format!("{}", b),

            Expression::Identifier(id)      => id.unparse(scope),
            Expression::ArrayIndex(id, idx) => format!("{}[{}]", id.unparse(scope), idx),

            Expression::Operation { lhs, op, rhs } => format!("{} {} {}",
                lhs.unparse(scope),
                op.unparse(scope),
                rhs.unparse(scope),
            ),
        }
    }
}

impl Unparse for IdentifierOrPointer {
    fn unparse(self, scope: &Scope) -> String {
        match self {
            IdentifierOrPointer::Identifier(ident) => ident.unparse(scope),

             // Look-up the pointer - if it has a name, use the name instead
            IdentifierOrPointer::Pointer(ptr) => match scope.lookup_ptr(ptr) {
                Some(name) => name.to_string(),
                None       => format!("0x{:X}", ptr),
            },
        }
    }
}

impl Unparse for Trigger {
    fn unparse(self, scope: &Scope) -> String {
        match self {
            Trigger::FloorTouch(obj)    => format!("floortouch {}", obj.unparse(scope)),
            Trigger::FloorAbove(obj)    => format!("floorabove {}", obj.unparse(scope)),
            Trigger::FloorInteract(obj) => format!("floorinteract {}", obj.unparse(scope)),
            Trigger::FloorJump(obj)     => format!("floorjump {}", obj.unparse(scope)),

            Trigger::WallTouch(obj)     => format!("walltouch {}", obj.unparse(scope)),
            Trigger::WallPush(obj)      => format!("wallpush {}", obj.unparse(scope)),
            Trigger::WallInteract(obj)  => format!("wallinteract {}", obj.unparse(scope)),
            Trigger::WallHammer(obj)    => format!("wallhammer {}", obj.unparse(scope)),

            Trigger::CeilingTouch(obj)  => format!("ceilingtouch {}", obj.unparse(scope)),
            Trigger::Bomb(ptr)          => format!("bomb {}", ptr.unparse(scope)),
            Trigger::FlagChange(ident)  => format!("flagchange {}", ident.unparse(scope)),
        }
    }
}

impl Unparse for TriggerObj {
    fn unparse(self, _: &Scope) -> String {
        match self {
            TriggerObj::Collider(id) => format!("{{collider:{}}}", id),
            TriggerObj::Entity(id)   => format!("{{entity:{}}}", id),
        }
    }
}

impl Unparse for Operator {
    fn unparse(self, _: &Scope) -> String {
        match self {
            Operator::Add => "+".to_string(),
            Operator::Sub => "-".to_string(),
            Operator::Mul => "*".to_string(),
            Operator::Div => "/".to_string(),
            Operator::Mod => "%".to_string(),

            Operator::Eq  => "==".to_string(),
            Operator::Ne  => "!=".to_string(),
            Operator::Lt  => "<".to_string(),
            Operator::Gt  => ">".to_string(),
            Operator::Lte => ">=".to_string(),
            Operator::Gte => "<=".to_string(),

            Operator::BitAndZ  => "&".to_string(),
            Operator::BitAndNz => "!&".to_string(),

            Operator::And => "and".to_string(),
            Operator::Or  => "or".to_string(),
            Operator::Not => "not".to_string(),
        }
    }
}

impl Unparse for Identifier {
    fn unparse(self, _: &Scope) -> String {
        self.0
    }
}

impl Unparse for DataType {
    fn unparse(self, _: &Scope) -> String {
        format!("{}", self)
    }
}

// Block
impl Unparse for Vec<Statement> {
    fn unparse(self, scope: &Scope) -> String {
        self
            .into_iter()
            .map(|stmt| stmt.unparse(scope))
            .join("\n")
    }
}

// Argument list
impl Unparse for Vec<RefCell<Expression>> {
    fn unparse(self, scope: &Scope) -> String {
        self
            .into_iter()
            .map(|arg| arg.into_inner().unparse(scope))
            .join(", ")
    }
}
