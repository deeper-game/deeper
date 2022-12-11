use std::collections::BTreeSet;
use std::collections::BTreeMap;

pub enum StatusEffect {
    Fire,
    Poison,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable(String);

pub enum Expr {
    Var(Variable),
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    ShiftLeft(Box<Expr>, Box<Expr>),
    ShiftRight(Box<Expr>, Box<Expr>),
}

pub enum Spec {
    Assign(Variable, Expr),
    Block(Vec<Spec>),
    For(Variable, Expr, Expr, Box<Spec>),
    If(Expr, Box<Spec>, Box<Spec>),
    Return(Variable),
}

pub type Value = u32;

pub type Context = BTreeMap<Variable, Value>;

pub fn interpret_expr(expr: &Expr, context: &Context) -> Value {
    match expr {
        Expr::Var(ref var) =>
            *(context.get(var).expect("interpret_expr: undefined variable")),
        Expr::Or(ref x, ref y) =>
            interpret_expr(x, &context) | interpret_expr(y, &context),
        Expr::And(ref x, ref y) =>
            interpret_expr(x, &context) & interpret_expr(y, &context),
        Expr::Xor(ref x, ref y) =>
            interpret_expr(x, &context) ^ interpret_expr(y, &context),
        Expr::Not(ref x) => !interpret_expr(x, &context),
        Expr::ShiftLeft(ref x, ref y) =>
            interpret_expr(x, &context) << interpret_expr(y, &context),
        Expr::ShiftRight(ref x, ref y) =>
            interpret_expr(x, &context) >> interpret_expr(y, &context),
    }
}

pub fn interpret_spec(spec: &Spec, context: &mut Context) -> Result<(), Value> {
    match spec {
        Spec::Assign(ref var, ref expr) => {
            context.insert(var.clone(), interpret_expr(expr, context));
        },
        Spec::Block(specs) => {
            for s in specs {
                interpret_spec(&s, context)?;
            }
        },
        Spec::For(ref variable, ref lower_expr, ref upper_expr, ref s) => {
            let lower = interpret_expr(lower_expr, context);
            let upper = interpret_expr(upper_expr, context);
            let shadowed: Option<Value> = context.get(variable).cloned();
            for i in lower .. upper + 1 {
                context.insert(variable.clone(), i);
                interpret_spec(s, context)?;
            }
            if let Some(x) = shadowed {
                context.insert(variable.clone(), x);
            } else {
                context.remove(variable);
            }
        },
        Spec::If(ref cond_expr, ref if_true, ref if_false) => {
            let cond = interpret_expr(cond_expr, context);
            if cond == 0 {
                interpret_spec(if_false, context)?;
            } else {
                interpret_spec(if_true, context)?;
            }
        },
        Spec::Return(ref variable) => {
            return Err(*(context.get(variable).unwrap()));
        },
    }
    Ok(())
}

pub enum LaneOp {
    Or,
    And,
    Xor,
    Not,
    Shift,
    Add,
    Negate,
    Subtract,
    Multiply,
    Divide,
}

pub enum CrossLaneOp {
    Rotate,
    AndReduce,
    OrReduce,
    AddReduce,
    XorReduce,
}

pub struct Bundle {
    lane_ops: Vec<LaneOp>,
    cross_lane_op: CrossLaneOp,
}

pub struct VLIW {
    pub number_of_registers: usize,
    pub lane_ops: BTreeSet<LaneOp>,
    pub cross_lane_ops: BTreeSet<CrossLaneOp>,
}
