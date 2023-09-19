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

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Instruction {
    Constant(usize),
    ConstantNum(f64),
    Return,
    Negate,
    Add,
    Substract,
    Multiply,
    Divide,
    Random,
    Print,
    Echo,
    Num,
    Not,
    Bool,
    Equal,
    Greater,
    Less,
    Max,
    Min,
    Floor,
    Ceil,
    Abs,
    Decr,
    Incr,
    Sin,
    Cos,
    Tan,
    Inv,
    Str,
    Upper,
    Lower,
    Trim,
    GreaterOrEqual,
    LessOrEqual,
    AlmostEqual,
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

    pub fn add_constant(&mut self, ast_node_idx: usize, value: Value) {
        self.constants.push(value);
        self.code.push(Instruction::Constant(self.constants.len()-1));
        self.ast_map.push(ast_node_idx);
    }

    pub fn read_constant(&self, index: usize) -> Value {
        self.constants[index]
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
}
