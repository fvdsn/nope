use rand::Rng;
use crate::parser::Parser;
use crate::parser::AstNode;
use std::time::SystemTime;

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
    Return,
    Negate,
    Add,
    Substract,
    Multiply,
    Divide,
    Random,
    Print,
    Num,
    Not,
    Bool,
    Equal,
    Greater,
    Less,
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
    chunk: Chunk,
    stack: Vec<Value>,
    ip: usize,
    rng: rand::rngs::ThreadRng,
}

impl Vm {
    pub fn new () -> Vm {
        return Vm {
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

    pub fn interpret(&mut self, code: String) -> InterpretResult {
        println!("create parser...");
        let mut parser = Parser::new(code);

        parser.parse();

        if parser.failed() {
            parser.print_errors();
            return InterpretResult::CompileError;
        }

        println!("compile...");
        if !self.compile(&parser.ast) {
            println!("compilation error");
            self.chunk.pretty_print();
            return InterpretResult::CompileError
        }

        println!("run...\n");
        
        let now = SystemTime::now();
        let res = self.run();

        match now.elapsed() {
            Ok(elapsed) => {
                println!("\n Ran in {}s", elapsed.as_secs());
            }
            _ => {
                println!("wtf");
            }
        };

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
                    "neg" => { self.chunk.push_op(node_idx, OpCode::Negate) },
                    "random" => { self.chunk.push_op(node_idx, OpCode::Random) },
                    "print" => { self.chunk.push_op(node_idx, OpCode::Print) },
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
                OpCode::Constant(cst_idx) => {
                    let cst = self.chunk.constants[cst_idx];
                    self.push(cst);
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
                OpCode::Random => {
                    let val: f64 = self.rng.gen();
                    self.push(Value::Num(val));
                },
            }
        }
    }
}

