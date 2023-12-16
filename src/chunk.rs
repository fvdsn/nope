use std::collections::HashMap;

use crate::{
    gc::GcRef,
};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Value {
    Null,
    Void,
    Boolean(bool),
    Num(f64),
    String(GcRef<String>),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Void => false,
            Value::Boolean(value) => *value,
            Value::Num(num) => *num != 0.0,
            Value::String(_) => true,
            // _ => true,
        }
    }
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Num(num) => *num == 0.0,
            _ => false,
        }
    }
    pub fn is_nullish(&self) -> bool {
        match self {
            Value::Null => true,
            Value::Void => true,
            _ => false,
        }
    }
    pub fn num_equiv(&self) -> f64 {
        match self {
            Value::Null => 0.0,
            Value::Void => 0.0,
            Value::Boolean(value) => (*value as i32) as f64,
            Value::Num(num) => *num,
            Value::String(_) => f64::NAN,
        }
    }
}

pub type GlobalsTable = HashMap<GcRef<String>, Value>;

#[derive(PartialEq, Debug, Clone)]
pub struct Local {
    name: String,
    depth: usize,
}


#[derive(PartialEq, Debug, Clone)]
pub struct LocalsTable {
    locals: Vec<Local>,
}

impl LocalsTable {
    pub fn new() -> LocalsTable {
        return LocalsTable {
            locals: vec![],
        };
    }
    pub fn push_anonymous(&mut self) {
        self.add_local("".to_owned())
    }
    pub fn add_local(&mut self, name: String) {
        self.locals.push(Local {depth: self.locals.len(), name: name.to_owned()});
    }
    pub fn pop(&mut self) {
        if self.locals.is_empty() {
            panic!("empty locals stash (pop)");
        }
        self.locals.pop();
    }
    pub fn get_local_depth(&self, name: &str) -> usize {
        if self.locals.is_empty() {
            panic!("empty locals stash (get)");
        }
        let mut i = self.locals.len() - 1;
        loop {
            if self.locals[i].name == name {
                return self.locals[i].depth;
            }
            if i == 0 {
                panic!("local not found: {}", name);
            } else {
                i = i - 1;
            }
        }
    }
    pub fn get_locals_count(&self) -> usize {
        return self.locals.len();
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Loop {
    pub locals_count: usize,
    pub continue_ip: usize,
    pub break_ip: usize,
}

#[derive(PartialEq, Debug, Clone)]
pub struct LoopsTable {
    loops: Vec<Loop>,
}

impl LoopsTable {
    pub fn new() -> LoopsTable {
        return LoopsTable {
            loops: vec![],
        };
    }
    pub fn push_loop(&mut self, locals_count: usize, continue_ip: usize, break_ip: usize) {
        self.loops.push(Loop { locals_count, continue_ip, break_ip });
    }
    pub fn pop_loop(&mut self) {
        self.loops.pop();
    }
    pub fn in_loop(&self) -> bool {
        return self.loops.len() > 0;
    }
    pub fn cur_loop(&self) -> Loop {
        return self.loops[self.loops.len()-1];
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Instruction {
    Constant(usize),
    PushNum(f64),
    PushVoid,
    PushNull,
    PushBool(bool),
    DefineGlobal(usize),
    GetGlobal(usize),
    SetGlobal(usize),
    LoadFromStack(usize),
    SetInStack(usize),
    Jump(i64),
    JumpIfFalse(i64),
    JumpIfTrue(i64),
    JumpIfNotNullish(i64),
    JumpIfNotZero(i64),
    IsVoid,
    IsNull,
    IsBool,
    IsNum,
    IsStr,
    IsNaN,
    IsInt,
    Swap,
    Pop,
    Return,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
    Random,
    Print,
    Echo,
    Num,
    ParseNum,
    Not,
    Bool,
    Equal,
    Greater,
    Less,
    BitwiseNot,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseLeftShift,
    BitwiseRightShift,
    BitwiseZeroRightShift,
    I32Add,
    I32Subtract,
    I32Multiply,
    I32Divide,
    Max,
    Min,
    Floor,
    Ceil,
    Abs,
    Decr,
    Incr,
    Sin,
    Cos,
    Acos,
    Tan,
    Inv,
    Acosh,
    Sinh,
    Asin,
    Asinh,
    Cosh,
    Tanh,
    Atan,
    Atanh,
    Atan2,
    Log2,
    Log10,
    Ln1p,
    Ln,
    Exp,
    Expm1,
    Sqrt,
    Cbrt,
    Round,
    Fround,
    Trunc,
    Sign,
    Str,
    Upper,
    Lower,
    Trim,
    JoinPaths,
    ReadTextFileSync,
    WriteTextFileSync,
    GreaterOrEqual,
    LessOrEqual,
    AlmostEqual,
    Replace,
    FromUnit,
    ToUnit,
    Silence,
    Bitstr,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub ast_map: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Chunk {
        return Chunk {
            code: vec![],
            constants: vec![],
            ast_map: vec![],
        };
    }

    pub fn add_constant(&mut self, value: Value) -> usize{
        self.constants.push(value);
        return self.constants.len() - 1;
    }

    pub fn write_constant(&mut self, ast_node_idx: usize, value: Value) -> usize{
        let cst_idx = self.add_constant(value);
        self.code.push(Instruction::Constant(cst_idx));
        self.ast_map.push(ast_node_idx);
        return cst_idx;
    }

    pub fn last_instr_idx(&self) -> usize {
        if self.code.is_empty() {
            0
        } else {
            self.code.len()-1
        }
    }

    pub fn read_constant(&self, index: usize) -> Value {
        self.constants[index]
    }

    pub fn read_constant_string(&self, index: usize) -> GcRef<String>{
        if let Value::String(s) = self.read_constant(index) {
            s
        } else {
            panic!("Constant is not a String");
        }
    }

    pub fn write(&mut self, ast_node_idx: usize, op: Instruction) {
        self.code.push(op);
        self.ast_map.push(ast_node_idx);
    }

    pub fn rewrite(&mut self, instr_idx: usize, op: Instruction) {
        self.code[instr_idx] = op;
    }

    pub fn pretty_print(&self) {
        for (idx, op) in self.code.iter().enumerate() {
            match op {
                Instruction::Constant(cst_idx) => {
                    let cst = self.constants[*cst_idx];
                    println!("{: <8} Constant {:?}", idx, cst);
                },
                _ => {
                    println!("{: <8} {:?}", idx, op);
                },
            };
        }
    }

    pub fn is_last_instruction_echo_or_print(&self) -> bool {
        if self.code.is_empty() {
            return false;
        } else {
            return matches!(self.code[self.code.len()-1], Instruction::Echo) ||
                   matches!(self.code[self.code.len()-1], Instruction::Print);
        }
    }
}
