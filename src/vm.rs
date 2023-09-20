use rand::Rng;
use std::time::SystemTime;
use std::path::Path;
use crate::{
    parser::{
        Parser,
        AstNode,
    },
    config::NopeConfig,
    chunk::{
        Value,
        Chunk,
        Instruction,
    },
    gc::{
        Gc,
        GcRef,
    },
};

use colored::*;


pub enum InterpretResult {
    Ok,
    CompileError,
    _RuntimeError,
}

pub struct Vm {
    config: NopeConfig,
    gc: Gc,
    chunk: Chunk,
    stack: Vec<Value>,
    ip: usize,
    rng: rand::rngs::ThreadRng,
}

impl Vm {
    pub fn new (config: NopeConfig) -> Vm {
        return Vm {
            gc: Gc::new(),
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

    fn intern(&mut self, name: String) -> GcRef<String> {
    //    self.mark_and_sweep();
        self.gc.intern(name)
    }

    fn value_to_str(&self, val: &Value) -> String {
        match val {
            Value::Num(num) =>  format!("{}", num),
            Value::Null => format!("null"),
            Value::Void => format!("_"),
            Value::Boolean(val) => {
                if *val { 
                    "true".to_string() 
                } else {
                    "false".to_string()
                }
            },
            Value::String(str_ref) => {
                let val = self.gc.deref(*str_ref);
                val.to_string() 
            },
        }
    }

    fn value_to_repr(&self, val: &Value) -> String {
        match val {
            Value::Num(num) =>  format!("{}", num),
            Value::Null => format!("null"),
            Value::Void => format!("_"),
            Value::Boolean(val) => {
                if *val { 
                    "true".to_string() 
                } else {
                    "false".to_string()
                }
            },
            Value::String(str_ref) => {
                let val = self.gc.deref(*str_ref);
                format!("\"{}\"", val.replace("\"", "\\\""))
            },
        }
    }

    fn print_val(&self, val: &Value) {
        println!("{}", self.value_to_str(val))
    }

    fn echo_val(&self, val: &Value) {
        match val {
            Value::Void => {
                println!();
            }
            _ => {
                println!();
                println!("   {}", self.value_to_repr(val).blue());
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
                self.chunk.add_constant(node_idx, Value::Num(*num));
            },
            AstNode::Null(_) => {
                self.chunk.add_constant(node_idx, Value::Null);
            },
            AstNode::Void(_) => {
                self.chunk.add_constant(node_idx, Value::Void);
            },
            AstNode::Boolean(_, val) => {
                self.chunk.add_constant(node_idx, Value::Boolean(*val));
            },
            AstNode::String(_, val) => {
                let str_ref = self.gc.intern(val.to_owned()); //FIXME should be self.intern ?
                self.chunk.add_constant(node_idx, Value::String(str_ref));
            },
            AstNode::FunctionCall(_, name, args) => {
                for arg in args {
                    if !self.compile_node(ast, *arg) {
                        println!("error compiling function {}", name);
                        return false;
                    }
                }
                match name.as_str() {
                    "add" => { self.chunk.write(node_idx, Instruction::Add) },
                    "sub" => { self.chunk.write(node_idx, Instruction::Substract) },
                    "mult" => { self.chunk.write(node_idx, Instruction::Multiply) },
                    "div" => { self.chunk.write(node_idx, Instruction::Divide) },
                    "min" => { self.chunk.write(node_idx, Instruction::Min) },
                    "max" => { self.chunk.write(node_idx, Instruction::Max) },
                    "neg" => { self.chunk.write(node_idx, Instruction::Negate) },
                    "abs" => { self.chunk.write(node_idx, Instruction::Abs) },
                    "floor" => { self.chunk.write(node_idx, Instruction::Floor) },
                    "ceil" => { self.chunk.write(node_idx, Instruction::Ceil) },
                    "decr" => { self.chunk.write(node_idx, Instruction::Decr) },
                    "incr" => { self.chunk.write(node_idx, Instruction::Incr) },
                    "sin" => { self.chunk.write(node_idx, Instruction::Sin) },
                    "cos" => { self.chunk.write(node_idx, Instruction::Cos) },
                    "tan" => { self.chunk.write(node_idx, Instruction::Tan) },
                    "inv" => { self.chunk.write(node_idx, Instruction::Inv) },
                    "random" => { self.chunk.write(node_idx, Instruction::Random) },
                    "print" => { self.chunk.write(node_idx, Instruction::Print) },
                    "echo" => { self.chunk.write(node_idx, Instruction::Echo) },
                    "num" => { self.chunk.write(node_idx, Instruction::Num) },
                    "not" => { self.chunk.write(node_idx, Instruction::Not) },
                    "bool" => { self.chunk.write(node_idx, Instruction::Bool) },
                    "str" => { self.chunk.write(node_idx, Instruction::Str) },
                    "join-paths" => { self.chunk.write(node_idx, Instruction::JoinPaths) },
                    "read-text" => { self.chunk.write(node_idx, Instruction::ReadTextFileSync) },
                    "write-text" => { self.chunk.write(node_idx, Instruction::WriteTextFileSync) },
                    "upper" => { self.chunk.write(node_idx, Instruction::Upper) },
                    "lower" => { self.chunk.write(node_idx, Instruction::Lower) },
                    "trim" => { self.chunk.write(node_idx, Instruction::Trim) },
                    "replace" => { self.chunk.write(node_idx, Instruction::Replace) },
                    "==" => { self.chunk.write(node_idx, Instruction::Equal) },
                    ">" => { self.chunk.write(node_idx, Instruction::Greater) },
                    "<" => { self.chunk.write(node_idx, Instruction::Less) },
                    ">=" => { self.chunk.write(node_idx, Instruction::GreaterOrEqual) },
                    "<=" => { self.chunk.write(node_idx, Instruction::LessOrEqual) },
                    "~=" => { self.chunk.write(node_idx, Instruction::AlmostEqual) },
                    "!=" => { 
                        self.chunk.write(node_idx, Instruction::Equal);
                        self.chunk.write(node_idx, Instruction::Not);
                    },
                    "flip-coin" => { 
                        self.chunk.write(node_idx, Instruction::Random);
                        self.chunk.write(node_idx, Instruction::ConstantNum(0.5));
                        self.chunk.write(node_idx, Instruction::GreaterOrEqual);
                    },
                    "rand100" => { 
                        self.chunk.write(node_idx, Instruction::Random);
                        self.chunk.write(node_idx, Instruction::ConstantNum(100.0));
                        self.chunk.write(node_idx, Instruction::Multiply);
                        self.chunk.write(node_idx, Instruction::Floor);
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
            if self.config.echo_result {
                self.chunk.write(self.chunk.ast_map[self.chunk.ast_map.len()-1], Instruction::Echo);
            }
            self.chunk.write(self.chunk.ast_map[self.chunk.ast_map.len()-1], Instruction::Return);
        } else {
            self.chunk.write(0, Instruction::Return);
        }
        return true;
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            let instr = self.chunk.code[self.ip];
            self.ip += 1;
            match instr {
                Instruction::Return => {
                    //println!("{:?}", self.pop());
                    return InterpretResult::Ok;
                },
                Instruction::Print=> {
                    self.print_val(&self.stack[self.stack.len() - 1]);
                },
                Instruction::Echo=> {
                    self.echo_val(&self.stack[self.stack.len() - 1]);
                },
                Instruction::Constant(cst_idx) => {
                    let cst = self.chunk.constants[cst_idx];
                    self.push(cst);
                },
                Instruction::ConstantNum(num)  => {
                    self.push(Value::Num(num));
                },
                Instruction::Num => {
                    let val = self.pop();
                    self.push(Value::Num(val.num_equiv()));
                },
                Instruction::Negate => {
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
                Instruction::Abs => {
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
                Instruction::Floor => {
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
                Instruction::Ceil => {
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
                Instruction::Incr => {
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
                Instruction::Decr => {
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
                Instruction::Sin => {
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
                Instruction::Cos => {
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
                Instruction::Tan => {
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
                Instruction::Inv => {
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
                Instruction::Not => {
                    let val = self.pop();
                    self.push(Value::Boolean(!val.is_truthy()));
                },
                Instruction::Bool => {
                    let val = self.pop();
                    self.push(Value::Boolean(val.is_truthy()));
                },
                Instruction::Str => {
                    let val = self.pop();
                    match &val {
                        Value::String(_) => {
                            self.push(val);
                        }
                        _ => {
                            let str_val = self.value_to_str(&val);
                            let ref_val = self.intern(str_val);
                            self.push(Value::String(ref_val));
                        }
                    }
                },
                Instruction::Upper => {
                    let val = self.pop();
                    match &val {

                        Value::String(ref_val) => {
                            let str_val = self.gc.deref(*ref_val).to_uppercase();
                            let ref_val = self.intern(str_val);
                            self.push(Value::String(ref_val));
                        }
                        _ => {
                            self.push(val);
                        }
                    }
                },
                Instruction::Lower => {
                    let val = self.pop();
                    match &val {

                        Value::String(ref_val) => {
                            let str_val = self.gc.deref(*ref_val).to_lowercase();
                            let ref_val = self.intern(str_val);
                            self.push(Value::String(ref_val));
                        }
                        _ => {
                            self.push(val);
                        }
                    }
                },
                Instruction::Trim => {
                    let val = self.pop();
                    match &val {

                        Value::String(ref_val) => {
                            let str_val = self.gc.deref(*ref_val).trim();
                            let ref_val = self.intern(str_val.to_owned());
                            self.push(Value::String(ref_val));
                        }
                        _ => {
                            self.push(val);
                        }
                    }
                },
                Instruction::ReadTextFileSync=> {
                    let val = self.pop();
                    let str_val = self.value_to_str(&val);
                    let txt = std::fs::read_to_string(Path::new(&str_val));
                    match txt {
                        Ok(txt_str) => {
                            let ref_txt = self.intern(txt_str);
                            self.push(Value::String(ref_txt));
                        },
                        Err(e) => {
                            let ref_err = self.intern(e.to_string());
                            self.push(Value::String(ref_err));
                        }
                    }
                },
                Instruction::WriteTextFileSync=> {
                    let text = self.pop();
                    let str_text = self.value_to_str(&text);
                    let path = self.pop();
                    let str_path = self.value_to_str(&path);
                    let res = std::fs::write(Path::new(&str_path), str_text);
                    match res {
                        Ok(_) => {
                            self.push(Value::Void);
                        },
                        Err(e) => {
                            let ref_err = self.intern(e.to_string());
                            self.push(Value::String(ref_err));
                        }
                    }
                },
                Instruction::Replace=> {
                    let text = self.pop();
                    let str_text = self.value_to_str(&text);
                    let repl_to = self.pop();
                    let str_repl_to = self.value_to_str(&repl_to);
                    let repl_from = self.pop();
                    let str_repl_from = self.value_to_str(&repl_from);
                    let res = str_text.replace(&str_repl_from, &str_repl_to);
                    let ref_res = self.intern(res);
                    self.push(Value::String(ref_res));
                },
                Instruction::Equal => {
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
                Instruction::Greater => {
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
                Instruction::GreaterOrEqual => {
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
                Instruction::Less => {
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
                Instruction::LessOrEqual => {
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
                Instruction::AlmostEqual => {
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
                Instruction::Add => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a + val_b));
                        },
                        (Value::String(ref_b), Value::String(ref_a)) => {
                            let str_a = self.gc.deref(ref_a);
                            let str_b = self.gc.deref(ref_b);
                            let str_ab = format!("{}{}", str_a, str_b);
                            let ref_ab = self.intern(str_ab);
                            self.push(Value::String(ref_ab));
                        }
                        (Value::String(ref_b), val_a) => {
                            let str_a = self.value_to_str(&val_a);
                            let str_b = self.gc.deref(ref_b);
                            let str_ab = format!("{}{}", str_a, str_b);
                            let ref_ab = self.intern(str_ab);
                            self.push(Value::String(ref_ab));
                        }
                        (val_b, Value::String(ref_a)) => {
                            let str_a = self.gc.deref(ref_a);
                            let str_b = self.value_to_str(&val_b);
                            let str_ab = format!("{}{}", str_a, str_b);
                            let ref_ab = self.intern(str_ab);
                            self.push(Value::String(ref_ab));
                        }
                        (b, a) => {
                            self.push(Value::Num(a.num_equiv() + b.num_equiv()));
                        },
                    }
                },
                Instruction::JoinPaths => {
                    let b = self.pop();
                    let a = self.pop();
                    let str_a = self.value_to_str(&a);
                    let str_b = self.value_to_str(&b);
                    let path_a = Path::new(&str_a);
                    let path_ab = path_a.join(str_b);
                    let str_ab = path_ab.to_string_lossy().to_string();
                    let ref_ab = self.intern(str_ab);
                    self.push(Value::String(ref_ab));
                },
                Instruction::Substract => {
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
                Instruction::Multiply => {
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
                Instruction::Divide => {
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
                Instruction::Min => {
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
                Instruction::Max => {
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
                Instruction::Random => {
                    let val: f64 = self.rng.gen();
                    self.push(Value::Num(val));
                },
            }
        }
    }
}

