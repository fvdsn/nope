use rand::Rng;
use crate::parser::Parser;
use crate::parser::AstNode;

#[derive(PartialEq, Debug, Clone, Copy)]
enum Value {
    Num(f64),
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
        }
    }

    pub fn interpret(&mut self, code: String) -> InterpretResult {
        let mut parser = Parser::new(code);

        parser.parse();

        if parser.has_errors() {
            parser.print_errors();
            return InterpretResult::CompileError;
        }

        if !self.compile(&parser.ast) {
            println!("compilation error");
            self.chunk.pretty_print();
            return InterpretResult::CompileError
        }

        return self.run();
    }

    fn compile_node(&mut self, ast: &Vec<AstNode>, node_idx: usize) -> bool {
        match &ast[node_idx] {
            AstNode::Number(_, num) => {
                self.chunk.push_constant(node_idx, Value::Num(*num));
            },
            AstNode::FunctionCall(_, name, args) => {
                for arg in args {
                    if !self.compile_node(ast, *arg) {
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
                    _ => { return false; }
                };
            },
            _ => {
                return false;
            }
        };
        return true;
    }

    pub fn compile(&mut self, ast: &Vec<AstNode>) -> bool {
        if !self.compile_node(ast, ast.len() - 1) {
            return false;
        }
        self.chunk.push_op(self.chunk.ast_map[self.chunk.ast_map.len()-1], OpCode::Return);
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
                OpCode::Negate => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(-num));
                        }
                    }
                },
                OpCode::Add => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a + val_b));
                        }
                    }
                },
                OpCode::Substract => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a - val_b));
                        }
                    }
                },
                OpCode::Multiply => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a * val_b));
                        }
                    }
                },
                OpCode::Divide => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a / val_b));
                        }
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

