use rand::Rng;
use std::time::SystemTime;
use crate::{
    parser::{
        Parser,
        AstNode,
    },
    config::NopeConfig,
};

use colored::*;

#[derive(PartialEq, Debug, Clone, Copy)]
enum Value {
    Null,
    Void,
    Boolean(bool),
    Num(f64),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Void => false,
            Value::Boolean(value) => *value,
            Value::Num(num) => *num != 0.0,
            // _ => true,
        }
    }
    pub fn num_equiv(&self) -> f64 {
        match self {
            Value::Null => 0.0,
            Value::Void => 0.0,
            Value::Boolean(value) => (*value as i32) as f64,
            Value::Num(num) => *num,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum OpCode {
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
    GreaterOrEqual,
    LessOrEqual,
    AlmostEqual,
}

#[derive(PartialEq, Debug, Clone)]
struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    ast_map: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Chunk {
        return Chunk {
            code: vec![],
            constants: vec![],
            ast_map: vec![],
        };
    }

    fn push_constant(&mut self, ast_node_idx: usize, value: Value) {
        self.constants.push(value);
        self.code.push(OpCode::Constant(self.constants.len()-1));
        self.ast_map.push(ast_node_idx);
    }

    fn push_op(&mut self, ast_node_idx: usize, op: OpCode) {
        self.code.push(op);
        self.ast_map.push(ast_node_idx);
    }

    pub fn pretty_print(&self) {
        for (idx, op) in self.code.iter().enumerate() {
            match op {
                OpCode::Constant(cst_idx) => {
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

pub enum InterpretResult {
    Ok,
    CompileError,
    _RuntimeError,
}

#[derive(Debug)]
pub struct Vm {
    config: NopeConfig,
    chunk: Chunk,
    stack: Vec<Value>,
    ip: usize,
    rng: rand::rngs::ThreadRng,
}

impl Vm {
    pub fn new (config: NopeConfig) -> Vm {
        return Vm {
            config,
            chunk: Chunk::new(),
            stack: vec![],
            ip: 0,
            rng: rand::thread_rng(),
        };
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Empty Stack")
    }

    fn print_val(&self, val: &Value) {
        match val {
            Value::Num(num) => { println!("{}", num); }
            Value::Null => { println!("null"); }
            Value::Void => { println!("_"); }
            Value::Boolean(val)=> { if *val { println!("true"); } else { println!("false")} }
        }
    }

    fn echo_val(&self, val: &Value) {
        match val {
            Value::Void => {
                println!();
            }
            _ => {
                println!();
                match val {
                    Value::Void => {},
                    Value::Num(num) => { println!("  {}", format!("{}", num).blue()); },
                    Value::Null => { println!("  {}", "null".blue()); },
                    Value::Boolean(val)=> {
                        println!("  {}",
                            if *val { "true".blue() } else { "false".blue() }
                        )
                    },
                };
                println!();
            }
        };
    }

    pub fn interpret(&mut self, code: String) -> InterpretResult {
        if self.config.debug {
            println!("create parser...");
        }
        
        let mut parser = Parser::new(self.config, code);

        parser.parse();

        if parser.failed() {
            parser.print_errors();
            return InterpretResult::CompileError;
        }
        
        if self.config.debug {
            println!("compile...");
        }

        if !self.compile(&parser.ast) {
            println!("compilation error");
            self.chunk.pretty_print();
            return InterpretResult::CompileError
        }

        if self.config.debug {
            println!("run...\n");
        }
        
        let now = SystemTime::now();
        let res = self.run();

        if self.config.debug {
            match now.elapsed() {
                Ok(elapsed) => {
                    println!("\n Ran in {}s", elapsed.as_secs());
                }
                _ => {
                    println!("wtf");
                }
            };
        }

        return res;
    }

    fn compile_node(&mut self, ast: &Vec<AstNode>, node_idx: usize) -> bool {
        match &ast[node_idx] {
            AstNode::Number(_, num) => {
                self.chunk.push_constant(node_idx, Value::Num(*num));
            },
            AstNode::Null(_) => {
                self.chunk.push_constant(node_idx, Value::Null);
            },
            AstNode::Void(_) => {
                self.chunk.push_constant(node_idx, Value::Void);
            },
            AstNode::Boolean(_, val) => {
                self.chunk.push_constant(node_idx, Value::Boolean(*val));
            },
            AstNode::FunctionCall(_, name, args) => {
                for arg in args {
                    if !self.compile_node(ast, *arg) {
                        println!("error compiling function {}", name);
                        return false;
                    }
                }
                match name.as_str() {
                    "add" => { self.chunk.push_op(node_idx, OpCode::Add) },
                    "sub" => { self.chunk.push_op(node_idx, OpCode::Substract) },
                    "mult" => { self.chunk.push_op(node_idx, OpCode::Multiply) },
                    "div" => { self.chunk.push_op(node_idx, OpCode::Divide) },
                    "min" => { self.chunk.push_op(node_idx, OpCode::Min) },
                    "max" => { self.chunk.push_op(node_idx, OpCode::Max) },
                    "neg" => { self.chunk.push_op(node_idx, OpCode::Negate) },
                    "abs" => { self.chunk.push_op(node_idx, OpCode::Abs) },
                    "floor" => { self.chunk.push_op(node_idx, OpCode::Floor) },
                    "ceil" => { self.chunk.push_op(node_idx, OpCode::Ceil) },
                    "decr" => { self.chunk.push_op(node_idx, OpCode::Decr) },
                    "incr" => { self.chunk.push_op(node_idx, OpCode::Incr) },
                    "sin" => { self.chunk.push_op(node_idx, OpCode::Sin) },
                    "cos" => { self.chunk.push_op(node_idx, OpCode::Cos) },
                    "tan" => { self.chunk.push_op(node_idx, OpCode::Tan) },
                    "inv" => { self.chunk.push_op(node_idx, OpCode::Inv) },
                    "random" => { self.chunk.push_op(node_idx, OpCode::Random) },
                    "print" => { self.chunk.push_op(node_idx, OpCode::Print) },
                    "echo" => { self.chunk.push_op(node_idx, OpCode::Echo) },
                    "num" => { self.chunk.push_op(node_idx, OpCode::Num) },
                    "not" => { self.chunk.push_op(node_idx, OpCode::Not) },
                    "bool" => { self.chunk.push_op(node_idx, OpCode::Bool) },
                    "==" => { self.chunk.push_op(node_idx, OpCode::Equal) },
                    ">" => { self.chunk.push_op(node_idx, OpCode::Greater) },
                    "<" => { self.chunk.push_op(node_idx, OpCode::Less) },
                    ">=" => { self.chunk.push_op(node_idx, OpCode::GreaterOrEqual) },
                    "<=" => { self.chunk.push_op(node_idx, OpCode::LessOrEqual) },
                    "~=" => { self.chunk.push_op(node_idx, OpCode::AlmostEqual) },
                    "!=" => { 
                        self.chunk.push_op(node_idx, OpCode::Equal);
                        self.chunk.push_op(node_idx, OpCode::Not);
                    },
                    "flip-coin" => { 
                        self.chunk.push_op(node_idx, OpCode::Random);
                        self.chunk.push_op(node_idx, OpCode::ConstantNum(0.5));
                        self.chunk.push_op(node_idx, OpCode::GreaterOrEqual);
                    },
                    "rand100" => { 
                        self.chunk.push_op(node_idx, OpCode::Random);
                        self.chunk.push_op(node_idx, OpCode::ConstantNum(100.0));
                        self.chunk.push_op(node_idx, OpCode::Multiply);
                        self.chunk.push_op(node_idx, OpCode::Floor);
                    },
                    _ => { 
                        println!("unknown function {}", name);
                        return false; 
                    }
                };
            },
            _ => {
                return false;
            }
        };
        return true;
    }

    pub fn compile(&mut self, ast: &Vec<AstNode>) -> bool {
        if !ast.is_empty() {
            if !self.compile_node(ast, ast.len() - 1) {
                return false;
            }
            self.chunk.push_op(self.chunk.ast_map[self.chunk.ast_map.len()-1], OpCode::Return);
        } else {
            self.chunk.push_op(0, OpCode::Return);
        }
        return true;
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            let instr = self.chunk.code[self.ip];
            self.ip += 1;
            match instr {
                OpCode::Return => {
                    //println!("{:?}", self.pop());
                    return InterpretResult::Ok;
                },
                OpCode::Print=> {
                    self.print_val(&self.stack[self.stack.len() - 1]);
                },
                OpCode::Echo=> {
                    self.echo_val(&self.stack[self.stack.len() - 1]);
                },
                OpCode::Constant(cst_idx) => {
                    let cst = self.chunk.constants[cst_idx];
                    self.push(cst);
                },
                OpCode::ConstantNum(num)  => {
                    self.push(Value::Num(num));
                },
                OpCode::Num => {
                    let val = self.pop();
                    self.push(Value::Num(val.num_equiv()));
                },
                OpCode::Negate => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(-num));
                        },
                        _ => {
                            self.push(Value::Num(f64::NAN));
                        },
                    }
                },
                OpCode::Abs => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(f64::abs(*num)));
                        },
                        _ => {
                            self.push(Value::Num(f64::abs(val.num_equiv())));
                        },
                    }
                },
                OpCode::Floor => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(f64::floor(*num)));
                        },
                        _ => {
                            self.push(Value::Num(f64::floor(val.num_equiv())));
                        },
                    }
                },
                OpCode::Ceil => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(f64::ceil(*num)));
                        },
                        _ => {
                            self.push(Value::Num(f64::ceil(val.num_equiv())));
                        },
                    }
                },
                OpCode::Incr => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(*num + 1.0));
                        },
                        _ => {
                            self.push(Value::Num(val.num_equiv() + 1.0));
                        },
                    }
                },
                OpCode::Decr => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(*num - 1.0));
                        },
                        _ => {
                            self.push(Value::Num(val.num_equiv() - 1.0));
                        },
                    }
                },
                OpCode::Sin => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(f64::sin(*num)));
                        },
                        _ => {
                            self.push(Value::Num(f64::sin(val.num_equiv())));
                        },
                    }
                },
                OpCode::Cos => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(f64::cos(*num)));
                        },
                        _ => {
                            self.push(Value::Num(f64::cos(val.num_equiv())));
                        },
                    }
                },
                OpCode::Tan => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(f64::tan(*num)));
                        },
                        _ => {
                            self.push(Value::Num(f64::tan(val.num_equiv())));
                        },
                    }
                },
                OpCode::Inv => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(1.0 / *num));
                        },
                        _ => {
                            self.push(Value::Num(1.0 / val.num_equiv()));
                        },
                    }
                },
                OpCode::Not => {
                    let val = self.pop();
                    self.push(Value::Boolean(!val.is_truthy()));
                },
                OpCode::Bool => {
                    let val = self.pop();
                    self.push(Value::Boolean(val.is_truthy()));
                },
                OpCode::Equal => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Boolean(val_a == val_b));
                        },
                        (Value::Boolean(val_b), Value::Boolean(val_a)) => {
                            self.push(Value::Boolean(val_a == val_b));
                        },
                        (Value::Null, Value::Null) => {
                            self.push(Value::Boolean(true));
                        },
                        (Value::Void, Value::Void) => {
                            self.push(Value::Boolean(true));
                        },
                        _ => {
                            self.push(Value::Boolean(false));
                        },
                    }
                },
                OpCode::Greater => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Boolean(val_a > val_b));
                        },
                        (b, a) => {
                            self.push(Value::Boolean(a.num_equiv() > b.num_equiv()));
                        },
                    }
                },
                OpCode::GreaterOrEqual => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Boolean(val_a >= val_b));
                        },
                        (b, a) => {
                            self.push(Value::Boolean(a.num_equiv() >= b.num_equiv()));
                        },
                    }
                },
                OpCode::Less => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Boolean(val_a < val_b));
                        },
                        (b, a) => {
                            self.push(Value::Boolean(a.num_equiv() < b.num_equiv()));
                        },
                    }
                },
                OpCode::LessOrEqual => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Boolean(val_a <= val_b));
                        },
                        (b, a) => {
                            self.push(Value::Boolean(a.num_equiv() <= b.num_equiv()));
                        },
                    }
                },
                OpCode::AlmostEqual => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Boolean(f64::abs(val_a - val_b) <= 0.00000001));
                        },
                        (b, a) => {
                            self.push(Value::Boolean(a.num_equiv() == b.num_equiv()));
                        },
                    }
                },
                OpCode::Add => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a + val_b));
                        },
                        (b, a) => {
                            self.push(Value::Num(a.num_equiv() + b.num_equiv()));
                        },
                    }
                },
                OpCode::Substract => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a - val_b));
                        }
                        (b, a) => {
                            self.push(Value::Num(a.num_equiv() - b.num_equiv()));
                        },
                    }
                },
                OpCode::Multiply => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a * val_b));
                        }
                        (b, a) => {
                            self.push(Value::Num(a.num_equiv() * b.num_equiv()));
                        },
                    }
                },
                OpCode::Divide => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a / val_b));
                        }
                        (b, a) => {
                            self.push(Value::Num(a.num_equiv() / b.num_equiv()));
                        },
                    }
                },
                OpCode::Min => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(f64::min(val_a, val_b)));
                        }
                        (b, a) => {
                            self.push(Value::Num(f64::min(a.num_equiv(), b.num_equiv())));
                        },
                    }
                },
                OpCode::Max => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(f64::max(val_a, val_b)));
                        }
                        (b, a) => {
                            self.push(Value::Num(f64::max(a.num_equiv(), b.num_equiv())));
                        },
                    }
                },
                OpCode::Random => {
                    let val: f64 = self.rng.gen();
                    self.push(Value::Num(val));
                },
            }
        }
    }
}

