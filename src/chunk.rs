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


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Instruction {
    Constant(usize),
    ConstantNum(f64),
    DefineGlobal(usize),
    GetGlobal(usize),
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
    Silence,
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
