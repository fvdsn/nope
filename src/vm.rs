use rand::Rng;
use std::time::SystemTime;
use std::path::Path;
use crate::{
    consts::EPSILON,
    parser::{
        Parser,
        AstNode,
        UnaryOperator,
        BinaryOperator,
    },
    units::{
        convert_unit_to_si,
        convert_si_to_unit,
    },
    penv::{
        Env,
    },
    stdlib::Stdlib,
    config::NopeConfig,
    chunk::{
        Value,
        Chunk,
        Instruction,
        GlobalsTable,
        LocalsTable,
        LoopsTable,
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
    parsers: Vec<Parser>,
    config: NopeConfig,
    gc: Gc,
    stdlib: Stdlib,
    globals: GlobalsTable,
    locals: LocalsTable,
    loops: LoopsTable,
    chunk: Chunk,
    stack: Vec<Value>,
    ip: usize,
    rng: rand::rngs::ThreadRng,
}

impl Vm {
    pub fn new (config: NopeConfig) -> Vm {
        return Vm {
            parsers: vec![],
            gc: Gc::new(),
            globals: GlobalsTable::new(),
            locals: LocalsTable::new(),
            loops: LoopsTable::new(),
            stdlib: Stdlib::new(),
            config,
            chunk: Chunk::new(),
            stack: vec![],
            ip: 0,
            rng: rand::thread_rng(),
        };
    }

    fn print_trace(&self) {
        println!("{:<4} {:<24} {:?}", self.ip, format!("{:?}", self.chunk.code[self.ip]), self.stack);
    }

    pub fn get_copy_of_last_env(&self) -> Option<Env> {
        if self.parsers.is_empty() {
            return None;
        } else {
            return Some(self.parsers[self.parsers.len() - 1].env.clone());
        }
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Empty Stack")
    }

    fn top(&mut self) -> Value {
        self.stack[self.stack.len()-1]
    }

    fn get_at_depth(&mut self, depth: usize) -> Value {
        self.stack[depth]
    }

    fn set_at_depth(&mut self, depth: usize, value: Value) {
        self.stack[depth] = value;
    }

    fn intern(&mut self, name: String) -> GcRef<String> {
    //    self.mark_and_sweep();
        self.gc.intern(name)
    }

    fn value_to_str(&self, val: &Value) -> String {
        match val {
            Value::Num(num) =>  format!("{}", num),
            Value::Null => "null".to_string(),
            Value::Void => "_".to_string(),
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
            Value::Null => "null".to_string(),
            Value::Void => "_".to_string(),
            Value::Boolean(val) => {
                if *val { 
                    "true".to_string() 
                } else {
                    "false".to_string()
                }
            },
            Value::String(str_ref) => {
                let val = self.gc.deref(*str_ref);
                format!("\"{}\"", val.replace('\"', "\\\""))
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
        
        let env = if let Some(env) = self.get_copy_of_last_env() {
            env
        } else {
            self.stdlib.make_env()
        };

        let mut parser = Parser::new_with_env(self.config, env, code);

        parser.parse();

        if parser.failed() {
            parser.print_errors();
            return InterpretResult::CompileError;
        }

        if self.config.debug {
            parser.env.print();
            parser.print();
            println!("compile...");
        }

        if !self.compile(&parser) {
            println!("compilation error");
            self.chunk.pretty_print();
            return InterpretResult::CompileError
        }

        self.parsers.push(parser);

        if self.config.debug || self.config.trace {
            self.chunk.pretty_print();
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

    fn compile_node(&mut self, ast: &Parser, node_idx: usize) -> bool {
        match &ast.ast[node_idx] {
            AstNode::Number(_, num) => {
                self.chunk.write(node_idx, Instruction::PushNum(*num));
            },
            AstNode::Null(_) => {
                self.chunk.write(node_idx, Instruction::PushNull);
            },
            AstNode::Void(_) => {
                self.chunk.write(node_idx, Instruction::PushVoid);
            },
            AstNode::Boolean(_, val) => {
                self.chunk.write(node_idx, Instruction::PushBool(*val));
            },
            AstNode::String(_, val) => {
                let str_ref = self.gc.intern(val.to_owned()); //FIXME should be self.intern ?
                self.chunk.write_constant(node_idx, Value::String(str_ref));
            },
            AstNode::Do(_, expr1, expr2) => {
                if !self.compile_node(ast, *expr1) {
                    println!("error compiling first expression of Do");
                    return false;
                }
                self.chunk.write(node_idx, Instruction::Pop);
                if !self.compile_node(ast, *expr2) {
                    println!("error compiling second expression of Do");
                    return false;
                }
            },
            AstNode::TopLevelBlock(_, expression_idx_list) => {
                for idx in expression_idx_list {
                    if !self.compile_node(ast, *idx) {
                        println!("error compiling code block");
                        return false;
                    }
                    if idx != expression_idx_list.last().unwrap() {
                        self.chunk.write(node_idx, Instruction::Pop);
                    }

                }
            },
            AstNode::GlobalLet(_, name, value_expr_node_idx, next_expr_node_idx) => {
                let name_ref = self.gc.intern(name.to_owned());
                let name_cst_idx = self.chunk.write_constant(node_idx, Value::String(name_ref));
                self.locals.push_anonymous();
                if !self.compile_node(ast, *value_expr_node_idx) {
                    println!("error compiling expression value for global variable {}", name);
                    return false;
                }
                self.chunk.write(node_idx, Instruction::DefineGlobal(name_cst_idx));
                self.locals.pop();
                if !self.compile_node(ast, *next_expr_node_idx) {
                    println!("error compile continuation expression for global variable {}", name);
                    return false;
                }
            },
            AstNode::GlobalSet(_, value_target_idx, value_expr_node_idx) => {
                let name = match ast.get_ast_node(*value_target_idx) {
                    AstNode::GlobalValueReference(_, name) => name,
                    _ => panic!("attempting to global set a non global var"),
                };
                let name_ref = self.gc.intern(name.to_owned());
                let name_cst_idx = self.chunk.write_constant(node_idx, Value::String(name_ref));
                if !self.compile_node(ast, *value_expr_node_idx) {
                    println!("error compiling expression value for global variable {}", name);
                    return false;
                }
                self.chunk.write(node_idx, Instruction::SetGlobal(name_cst_idx));
            },
            AstNode::GlobalValueReference(_, var_name) => {
                let name_ref = self.gc.intern(var_name.to_owned());
                let name_cst_idx = self.chunk.add_constant(Value::String(name_ref));
                self.chunk.write(node_idx, Instruction::GetGlobal(name_cst_idx));
            },
            AstNode::LocalLet(_, name, value_expr_node_idx, next_expr_node_idx) => {
                if !self.compile_node(ast, *value_expr_node_idx) {
                    println!("error compiling expression value for global variable {}", name);
                    return false;
                }
                self.locals.add_local(name.to_owned());
                if !self.compile_node(ast, *next_expr_node_idx) {
                    println!("error compile continuation expression for global variable {}", name);
                    return false;
                }
                self.locals.pop();
                self.chunk.write(node_idx, Instruction::Swap);
                self.chunk.write(node_idx, Instruction::Pop);
            },
            AstNode::LocalSet(_, value_target_idx, value_expr_node_idx) => {
                let name = match ast.get_ast_node(*value_target_idx) {
                    AstNode::LocalValueReference(_, name) => name,
                    _ => panic!("attempting to local set a non local var"),
                };
                let depth = self.locals.get_local_depth(&name);
                if !self.compile_node(ast, *value_expr_node_idx) {
                    println!("error compiling expression value for local variable {}", name);
                    return false;
                }
                self.chunk.write(node_idx, Instruction::SetInStack(depth));
            },
            AstNode::LocalValueReference(_, var_name) => {
                let depth = self.locals.get_local_depth(var_name);
                self.chunk.write(node_idx, Instruction::LoadFromStack(depth));
            },
            AstNode::IfElse(_, cond_expr_node_idx, val_expr_node_idx, else_expr_node_idx) => {
                if !self.compile_node(ast, *cond_expr_node_idx) {
                    println!("error compiling if condition");
                    return false;
                }
                self.chunk.write(node_idx, Instruction::JumpIfFalse(0));
                let jmp_to_else_idx = self.chunk.last_instr_idx();
                self.chunk.write(node_idx, Instruction::Pop);

                if !self.compile_node(ast, *val_expr_node_idx) {
                    println!("error compiling true branch of if block");
                    return false;
                }

                self.chunk.write(node_idx, Instruction::JumpIfFalse(0));
                let jmp_to_end_idx = self.chunk.last_instr_idx();

                self.chunk.write(node_idx, Instruction::Pop);
                let jmp_to_else_target_idx = self.chunk.last_instr_idx();

                if !self.compile_node(ast, *else_expr_node_idx) {
                    println!("error compiling false branch of if block");
                    return false;
                }

                let jmp_to_end_target_idx = self.chunk.last_instr_idx() + 1;

                self.chunk.rewrite(jmp_to_else_idx, Instruction::JumpIfFalse(
                    jmp_to_else_target_idx as i64 - jmp_to_else_idx as i64
                ));

                self.chunk.rewrite(jmp_to_end_idx, Instruction::Jump(
                    jmp_to_end_target_idx as i64 - jmp_to_end_idx as i64
                ));
            },
            AstNode::WhileLoop(_, cond_expr_node_idx, expr_node_idx) => {

                self.chunk.write(node_idx, Instruction::Jump(2));
                self.chunk.write(node_idx, Instruction::Jump(0));

                let break_idx = self.chunk.last_instr_idx();

                self.chunk.write(node_idx, Instruction::PushVoid);

                let idx_001 = self.chunk.last_instr_idx() + 1;

                if !self.compile_node(ast, *cond_expr_node_idx) {
                    println!("error compiling while condition");
                    return false;
                }
                self.chunk.write(node_idx, Instruction::JumpIfFalse(0));
                let jmp_to_999_idx = self.chunk.last_instr_idx();

                self.chunk.write(node_idx, Instruction::Pop);
                self.chunk.write(node_idx, Instruction::Pop);

                self.loops.push_loop(self.locals.get_locals_count(), idx_001, break_idx);

                if !self.compile_node(ast, *expr_node_idx) {
                    println!("error compiling while body");
                    return false;
                }

                self.loops.pop_loop();

                self.chunk.write(node_idx, Instruction::Jump(0));
                let jmp_to_001_idx = self.chunk.last_instr_idx();

                self.chunk.write(node_idx, Instruction::Pop);
                let idx_999 = self.chunk.last_instr_idx();

                self.chunk.rewrite(break_idx, Instruction::Jump(
                    (idx_999 + 1) as i64 - break_idx as i64
                ));

                self.chunk.rewrite(jmp_to_999_idx, Instruction::JumpIfFalse(
                    idx_999 as i64 - jmp_to_999_idx as i64
                ));

                self.chunk.rewrite(jmp_to_001_idx, Instruction::Jump(
                    idx_001 as i64 - jmp_to_001_idx as i64
                ));
            },
            AstNode::Continue(_) => {
                if !self.loops.in_loop() {
                    println!("error compiling 'continue', not in a loop");
                    return false;
                }

                let cloop = self.loops.cur_loop();

                let lcount = self.locals.get_locals_count();

                let var_to_pop = lcount - cloop.locals_count;
                for _ in 0..var_to_pop {
                    self.chunk.write(node_idx, Instruction::Pop);
                }
                self.chunk.write(node_idx, Instruction::PushVoid);
                self.chunk.write(node_idx, Instruction::Jump(
                    cloop.continue_ip as i64 - (self.chunk.last_instr_idx() + 1) as i64
                ));
            },
            AstNode::Break(_, expr_node_idx) => {
                if !self.loops.in_loop() {
                    println!("error compiling 'break', not in a loop");
                    return false;
                }

                let cloop = self.loops.cur_loop();

                let lcount = self.locals.get_locals_count();

                let var_to_pop = lcount - cloop.locals_count;
                for _ in 0..var_to_pop {
                    self.chunk.write(node_idx, Instruction::Pop);
                }
                if !self.compile_node(ast, *expr_node_idx) {
                    println!("error compiling break value");
                    return false;
                }
                self.chunk.write(node_idx, Instruction::Jump(
                    cloop.break_ip as i64 - (self.chunk.last_instr_idx() + 1) as i64
                ));
            },
            AstNode::FunctionCall(_, name, args) => {
                for arg in args {
                    if !self.compile_node(ast, *arg) {
                        println!("error compiling function {}", name);
                        return false;
                    }
                }
                match self.stdlib.get_function_instructions(name) {
                    Some(instructions) => {
                        for instruction in instructions {
                            self.chunk.write(node_idx, *instruction);
                        }
                    },
                    None => {
                        println!("error compiling function {}, not implemented", name);
                        return false;
                    }
                };
            },
            AstNode::UnaryOperator(_, op, expr_node_idx) => {
                if !self.compile_node(ast, *expr_node_idx) {
                    println!("error compiling value of unary expression");
                    return false;
                }
                match op {
                    UnaryOperator::Not => {
                        self.chunk.write(node_idx, Instruction::Not);
                    },
                    UnaryOperator::Negate => {
                        self.chunk.write(node_idx, Instruction::Negate);
                    },
                    UnaryOperator::Add => {
                        self.chunk.write(node_idx, Instruction::Num);
                    },
                    UnaryOperator::BitwiseNot=> {
                        self.chunk.write(node_idx, Instruction::BitwiseNot);
                    },
                }
            },
            AstNode::BinaryOperator(_, BinaryOperator::And, lexpr_node_idx, rexpr_node_idx) => {
                if !self.compile_node(ast, *lexpr_node_idx) {
                    println!("error compiling left condition of &&");
                    return false;
                }
                self.chunk.write(node_idx, Instruction::JumpIfFalse(0));
                let jmp_to_end_idx = self.chunk.last_instr_idx();
                self.chunk.write(node_idx, Instruction::Pop);

                if !self.compile_node(ast, *rexpr_node_idx) {
                    println!("error compiling right condition of &&");
                    return false;
                }

                let jmp_to_end_target_idx = self.chunk.last_instr_idx() + 1;

                self.chunk.rewrite(jmp_to_end_idx, Instruction::JumpIfFalse(
                    jmp_to_end_target_idx as i64 - jmp_to_end_idx as i64 
                ));
            },
            AstNode::BinaryOperator(_, BinaryOperator::Or, lexpr_node_idx, rexpr_node_idx) => {
                if !self.compile_node(ast, *lexpr_node_idx) {
                    println!("error compiling left condition of ||");
                    return false;
                }
                self.chunk.write(node_idx, Instruction::JumpIfTrue(0));
                let jmp_to_end_idx = self.chunk.last_instr_idx();
                self.chunk.write(node_idx, Instruction::Pop);

                if !self.compile_node(ast, *rexpr_node_idx) {
                    println!("error compiling right condition of ||");
                    return false;
                }

                let jmp_to_end_target_idx = self.chunk.last_instr_idx() + 1;

                self.chunk.rewrite(jmp_to_end_idx, Instruction::JumpIfTrue(
                    jmp_to_end_target_idx as i64 - jmp_to_end_idx as i64
                ));
            },
            AstNode::BinaryOperator(_, BinaryOperator::NullishOr, lexpr_node_idx, rexpr_node_idx) => {
                if !self.compile_node(ast, *lexpr_node_idx) {
                    println!("error compiling left condition of ||");
                    return false;
                }
                self.chunk.write(node_idx, Instruction::JumpIfNotNullish(0));
                let jmp_to_end_idx = self.chunk.last_instr_idx();
                self.chunk.write(node_idx, Instruction::Pop);

                if !self.compile_node(ast, *rexpr_node_idx) {
                    println!("error compiling right condition of ||");
                    return false;
                }

                let jmp_to_end_target_idx = self.chunk.last_instr_idx() + 1;

                self.chunk.rewrite(jmp_to_end_idx, Instruction::JumpIfNotNullish(
                    jmp_to_end_target_idx as i64 - jmp_to_end_idx as i64
                ));
            },
            AstNode::BinaryOperator(_, BinaryOperator::Repeat, cexpr_node_idx, vexpr_node_idx) => {
                if !self.compile_node(ast, *cexpr_node_idx) {
                    println!("error compiling count of *:");
                    return false;
                }
                self.chunk.write(node_idx, Instruction::Num);
                self.chunk.write(node_idx, Instruction::PushNum(0.0));
                self.chunk.write(node_idx, Instruction::Max);

                self.chunk.write(node_idx, Instruction::JumpIfNotZero(0));
                let idx_00a = self.chunk.last_instr_idx();

                self.chunk.write(node_idx, Instruction::Pop);
                self.chunk.write(node_idx, Instruction::PushVoid);

                self.chunk.write(node_idx, Instruction::Jump(0));

                let idx_00b = self.chunk.last_instr_idx();
                let idx_002 = self.chunk.last_instr_idx() + 1;

                self.chunk.rewrite(idx_00a, Instruction::JumpIfNotZero(
                    idx_002 as i64 - idx_00a as i64
                ));

                self.locals.push_anonymous();

                if !self.compile_node(ast, *vexpr_node_idx) {
                    println!("error compiling value of *:");
                    return false;
                }

                self.chunk.write(node_idx, Instruction::Swap);
                self.chunk.write(node_idx, Instruction::Decr);
                self.chunk.write(node_idx, Instruction::JumpIfNotZero(0));
                let idx_006 = self.chunk.last_instr_idx();
                self.chunk.write(node_idx, Instruction::Pop);
                self.chunk.write(node_idx, Instruction::Jump(0));
                let idx_00c = self.chunk.last_instr_idx();
                self.chunk.write(node_idx, Instruction::Swap);
                let idx_007 = self.chunk.last_instr_idx();

                self.chunk.rewrite(idx_006, Instruction::JumpIfNotZero(
                    idx_007 as i64 - idx_006 as i64
                ));

                self.locals.push_anonymous();

                if !self.compile_node(ast, *vexpr_node_idx) {
                    println!("error compiling value of *:");
                    return false;
                }

                self.locals.pop();
                self.locals.pop();

                self.chunk.write(node_idx, Instruction::Add);
                self.chunk.write(node_idx, Instruction::Swap);
                self.chunk.write(node_idx, Instruction::Decr);
                let idx_012 = self.chunk.last_instr_idx() + 1;
                self.chunk.write(node_idx, Instruction::JumpIfNotZero(
                    idx_007 as i64 - idx_012 as i64
                ));
                self.chunk.write(node_idx, Instruction::Pop);
                let idx_999 = self.chunk.last_instr_idx() + 1;

                self.chunk.rewrite(idx_00b, Instruction::Jump(
                    idx_999 as i64 - idx_00b as i64
                ));

                self.chunk.rewrite(idx_00c, Instruction::Jump(
                    idx_999 as i64 - idx_00c as i64
                ));
            },
            AstNode::BinaryOperator(_, op, lexpr_node_idx, rexpr_node_idx) => {
                if !self.compile_node(ast, *lexpr_node_idx) {
                    println!("error compiling left arm of binary operator");
                    return false;
                }
                if !self.compile_node(ast, *rexpr_node_idx) {
                    println!("error compiling right arm of binary operator");
                    return false;
                }
                match op {
                    BinaryOperator::Equal          => { self.chunk.write(node_idx, Instruction::Equal); },
                    BinaryOperator::NotEqual       => { 
                        self.chunk.write(node_idx, Instruction::Equal);
                        self.chunk.write(node_idx, Instruction::Not);
                    },
                    BinaryOperator::Less           => { self.chunk.write(node_idx, Instruction::Less);},
                    BinaryOperator::LessOrEqual    => { self.chunk.write(node_idx, Instruction::LessOrEqual);},
                    BinaryOperator::Greater        => { self.chunk.write(node_idx, Instruction::Greater);},
                    BinaryOperator::GreaterOrEqual => { self.chunk.write(node_idx, Instruction::GreaterOrEqual);},
                    BinaryOperator::AlmostEqual    => { self.chunk.write(node_idx, Instruction::AlmostEqual);},
                    BinaryOperator::NotAlmostEqual => {
                        self.chunk.write(node_idx, Instruction::AlmostEqual);
                        self.chunk.write(node_idx, Instruction::Not);
                    },
                    BinaryOperator::Add            => { self.chunk.write(node_idx, Instruction::Add);},
                    BinaryOperator::Subtract       => { self.chunk.write(node_idx, Instruction::Subtract);},
                    BinaryOperator::Multiply       => { self.chunk.write(node_idx, Instruction::Multiply);},
                    BinaryOperator::Divide         => { self.chunk.write(node_idx, Instruction::Divide);},
                    BinaryOperator::Modulo         => { self.chunk.write(node_idx, Instruction::Modulo);},
                    BinaryOperator::Power          => { self.chunk.write(node_idx, Instruction::Power);},
                    BinaryOperator::BitwiseAnd     => { self.chunk.write(node_idx, Instruction::BitwiseAnd);},
                    BinaryOperator::BitwiseOr      => { self.chunk.write(node_idx, Instruction::BitwiseOr);},
                    BinaryOperator::BitwiseXor     => { self.chunk.write(node_idx, Instruction::BitwiseXor);},
                    BinaryOperator::BitwiseLeftShift      => { self.chunk.write(node_idx, Instruction::BitwiseLeftShift);},
                    BinaryOperator::BitwiseRightShift     => { self.chunk.write(node_idx, Instruction::BitwiseRightShift);},
                    BinaryOperator::BitwiseZeroRightShift => { self.chunk.write(node_idx, Instruction::BitwiseZeroRightShift);},
                    BinaryOperator::I32Add         => { self.chunk.write(node_idx, Instruction::I32Add);},
                    BinaryOperator::I32Subtract    => { self.chunk.write(node_idx, Instruction::I32Subtract);},
                    BinaryOperator::I32Multiply    => { self.chunk.write(node_idx, Instruction::I32Multiply);},
                    BinaryOperator::I32Divide      => { self.chunk.write(node_idx, Instruction::I32Divide);},
                    BinaryOperator::And            => { panic!("BinaryOperator::And case should have be handled elsewhere") },
                    BinaryOperator::Or             => { panic!("BinaryOperator::Or case should have be handled elsewhere") },
                    BinaryOperator::NullishOr      => { panic!("BinaryOperator::NullishOr case should have be handled elsewhere") },
                    BinaryOperator::Repeat         => { panic!("BinaryOperator::Repeat case should have be handled elsewhere") },
                }
            },
            _ => {
                return false;
            }
        };
        return true;
    }

    pub fn compile(&mut self, parser:&Parser) -> bool {
        let ast: &Vec<AstNode> = &parser.ast;
        if !ast.is_empty() {
            if !self.compile_node(parser, ast.len() - 1) {
                return false;
            }
            if self.config.echo_result && !self.chunk.is_last_instruction_echo_or_print() {
                self.chunk.write(self.chunk.ast_map[self.chunk.ast_map.len()-1], Instruction::Echo);
            }
            self.chunk.write(0, Instruction::Pop);
            self.chunk.write(self.chunk.ast_map[self.chunk.ast_map.len()-1], Instruction::Return);
        } else {
            self.chunk.write(0, Instruction::Return);
        }
        return true;
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            if self.config.trace {
                self.print_trace();
            }
            // println!("ip:{}", self.ip);
            let instr = self.chunk.code[self.ip];
            self.ip += 1;
            match instr {
                Instruction::Return => {
                    //println!("{:?}", self.pop());
                    return InterpretResult::Ok;
                },
                Instruction::Pop => {
                    self.pop();
                },
                Instruction::Silence => {
                    self.pop();
                    self.push(Value::Void);
                },
                Instruction::Print=> {
                    self.print_val(&self.stack[self.stack.len() - 1]);
                },
                Instruction::Echo=> {
                    self.echo_val(&self.stack[self.stack.len() - 1]);
                },
                Instruction::Constant(cst_idx) => {
                    let cst = self.chunk.read_constant(cst_idx);
                    self.push(cst);
                },
                Instruction::PushNum(num)  => {
                    self.push(Value::Num(num));
                },
                Instruction::PushVoid  => {
                    self.push(Value::Void);
                },
                Instruction::PushNull  => {
                    self.push(Value::Null);
                },
                Instruction::PushBool(val)  => {
                    self.push(Value::Boolean(val));
                },
                Instruction::IsVoid  => {
                    let v = self.pop();
                    self.push(Value::Boolean(matches!(v, Value::Void)));
                },
                Instruction::IsNull => {
                    let v = self.pop();
                    self.push(Value::Boolean(matches!(v, Value::Null)));
                },
                Instruction::IsBool => {
                    let v = self.pop();
                    self.push(Value::Boolean(matches!(v, Value::Boolean(_))));
                },
                Instruction::IsNum => {
                    let v = self.pop();
                    self.push(Value::Boolean(matches!(v, Value::Num(_))));
                },
                Instruction::IsStr => {
                    let v = self.pop();
                    self.push(Value::Boolean(matches!(v, Value::String(_))));
                },
                Instruction::IsNaN => {
                    match self.pop() {
                        Value::Num(v) => self.push(Value::Boolean(v.is_nan())),
                        _ => self.push(Value::Boolean(false)),
                    }
                },
                Instruction::IsInt=> {
                    match self.pop() {
                        Value::Num(v) => self.push(Value::Boolean(v.fract() == 0.0)),
                        _ => self.push(Value::Boolean(false)),
                    }
                },
                Instruction::DefineGlobal(cst_idx)  => {
                    let global_name = self.chunk.read_constant_string(cst_idx);
                    let value = self.pop();
                    self.globals.insert(global_name, value);
                    self.pop();
                },
                Instruction::GetGlobal(cst_idx) => {
                    let global_name = self.chunk.read_constant_string(cst_idx);
                    match self.globals.get(&global_name) {
                        Some(&value) => self.push(value),
                        None => {
                            let global_name = self.gc.deref(global_name);
                            panic!("Undefined global {}", global_name);
                        }
                    }
                },
                Instruction::SetGlobal(cst_idx) => {
                    let global_name = self.chunk.read_constant_string(cst_idx);
                    let value = self.pop();
                    self.globals.insert(global_name, value);
                    self.pop();
                    self.push(value);
                },
                Instruction::LoadFromStack(depth) => {
                    let value = self.get_at_depth(depth);
                    self.push(value);
                },
                Instruction::SetInStack(depth) => {
                    let value = self.top();
                    self.set_at_depth(depth, value);
                },
                Instruction::Jump(offset) => {
                    self.ip = (self.ip as i64 + offset - 1) as usize;
                },
                Instruction::JumpIfFalse(offset) => {
                    if !self.top().is_truthy() {
                        self.ip = (self.ip as i64 + offset - 1) as usize;
                    }
                },
                Instruction::JumpIfNotNullish(offset) => {
                    if !self.top().is_nullish() {
                        self.ip = (self.ip as i64 + offset - 1) as usize;
                    }
                },
                Instruction::JumpIfNotZero(offset) => {
                    if !self.top().is_zero() {
                        self.ip = (self.ip as i64 + offset - 1) as usize;
                    }
                },
                Instruction::JumpIfTrue(offset) => {
                    if self.top().is_truthy() {
                        self.ip = (self.ip as i64 + offset - 1) as usize;
                    }
                },
                Instruction::Num => {
                    let val = self.pop();
                    self.push(Value::Num(val.num_equiv()));
                },
                Instruction::ParseNum => {
                    let val = self.pop();
                    match val {
                        Value::String(ref_val) => {
                            let str_val = self.gc.deref(ref_val);
                            match str_val.parse::<f64>() {
                                Ok(num) => {
                                    self.push(Value::Num(num))
                                },
                                Err(_) => {
                                    self.push(Value::Num(f64::NAN))
                                }
                            }
                        },
                        _ => {
                            self.push(Value::Num(val.num_equiv()));
                        }
                    }
                },
                Instruction::Len => {
                    let val = self.pop();
                    match val {
                        Value::String(ref_val) => {
                            let str_val = self.gc.deref(ref_val);
                            self.push(Value::Num(str_val.chars().count() as f64));
                        }
                        _ => {
                            self.push(Value::Num(0.0));
                        }
                    }
                }
                Instruction::SubStr => {
                    let ostr = self.pop();
                    let mut to_idx = self.pop().num_equiv() as i64;
                    let mut from_idx = self.pop().num_equiv() as i64;

                    match ostr {
                        Value::String(ref_val) => {
                            let str_val = self.gc.deref(ref_val);
                            let strlen = str_val.chars().count();
                            if strlen == 0 {
                                self.push(Value::String(ref_val));
                            } else {
                                from_idx = from_idx.min(strlen as i64);
                                from_idx = from_idx.max(-(strlen as i64) - 1);
                                to_idx = to_idx.min(strlen as i64);
                                to_idx = to_idx.max(-(strlen as i64) - 1);
                                if from_idx < 0 {
                                    from_idx += strlen as i64 + 1;
                                }
                                if to_idx < 0 {
                                    to_idx += strlen as i64 + 1;
                                }

                                if to_idx < from_idx {
                                    to_idx = from_idx;
                                }
                                if to_idx == from_idx {
                                    let s = self.intern("".to_owned());
                                    self.push(Value::String(s));
                                } else {
                                    let mut newstrc: Vec<char> = vec![];
                                    for (idx, c) in str_val.char_indices() {
                                        if idx as i64 >= to_idx {
                                            break;
                                        }
                                        if idx as i64 >= from_idx {
                                            newstrc.push(c);
                                        }
                                    }
                                    let s = self.intern(newstrc.iter().collect());
                                    self.push(Value::String(s));
                                }
                            }
                        },
                        _ => {
                            let s = self.intern("".to_owned());
                            self.push(Value::String(s));
                        }
                    }
                },
                Instruction::CharAt => {
                    let ostr = self.pop();
                    let mut idx = self.pop().num_equiv() as i64;

                    match ostr {
                        Value::String(ref_val) => {
                            let str_val = self.gc.deref(ref_val);
                            let strlen = str_val.chars().count() as i64;
                            if strlen == 0 {
                                self.push(Value::String(ref_val));
                            } else {
                                if idx >= strlen {
                                    self.push(Value::Void);
                                } else if idx <= -strlen - 1 {
                                    self.push(Value::Void);
                                } else {
                                    idx = idx.max(-(strlen as i64));
                                    if idx < 0 {
                                        idx += strlen
                                    }
                                    let c = str_val.chars().nth(idx as usize).unwrap();
                                    let s = self.intern(c.to_string());
                                    self.push(Value::String(s));
                                }
                            }
                        },
                        _ => {
                            let s = self.intern("".to_owned());
                            self.push(Value::String(s));
                        }
                    }
                },
                Instruction::Swap => {
                    let val1 = self.pop();
                    let val2 = self.pop();
                    self.push(val1);
                    self.push(val2);
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
                Instruction::Acos => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(f64::acos(*num)));
                        },
                        _ => {
                            self.push(Value::Num(f64::acos(val.num_equiv())));
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
                Instruction::BitwiseNot => {
                    let val = self.pop();
                    match &val {
                        Value::Num(num) => {
                            self.push(Value::Num(!(*num as i32) as f64));
                        },
                        _ => {
                            self.push(Value::Num(!(val.num_equiv() as i32) as f64));
                        },
                    }
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
                            self.push(Value::Boolean(f64::abs(val_a - val_b) <= EPSILON));
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
                Instruction::Subtract => {
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
                Instruction::Power => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a.powf(val_b)));
                        }
                        (b, a) => {
                            self.push(Value::Num(a.num_equiv().powf(b.num_equiv())));
                        },
                    }
                },
                Instruction::Modulo => {
                    let ops = (self.pop(), self.pop());
                    match ops {
                        (Value::Num(val_b), Value::Num(val_a)) => {
                            self.push(Value::Num(val_a % val_b));
                        }
                        (b, a) => {
                            self.push(Value::Num(a.num_equiv() % b.num_equiv()));
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
                Instruction::BitwiseAnd => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) & (b.num_equiv() as i32)) as f64));
                },
                Instruction::BitwiseOr => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) | (b.num_equiv() as i32)) as f64));
                },
                Instruction::BitwiseXor => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) ^ (b.num_equiv() as i32)) as f64));
                },
                Instruction::BitwiseLeftShift => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) << (b.num_equiv() as i32)) as f64));
                },
                Instruction::BitwiseRightShift => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) >> (b.num_equiv() as i32)) as f64));
                },
                Instruction::BitwiseZeroRightShift => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32 as u32) >> (b.num_equiv() as i32 as u32)) as i32 as f64));
                },
                Instruction::I32Add => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) + (b.num_equiv() as i32)) as f64));
                },
                Instruction::I32Subtract => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) - (b.num_equiv() as i32)) as f64));
                },
                Instruction::I32Multiply => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) * (b.num_equiv() as i32)) as f64));
                },
                Instruction::I32Divide => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Num(((a.num_equiv() as i32) / (b.num_equiv() as i32)) as f64));
                },
                Instruction::Bitstr => {
                    let val = self.pop().num_equiv() as i32;
                    let mut bitstr: Vec<char> = vec![];
                    for i in 0..32 {
                        let idx = 1 << (31-i);
                        if (val & idx) != 0 {
                            bitstr.push('1');
                        } else {
                            bitstr.push('0');
                        }
                    }

                    let ref_val = self.intern(bitstr.iter().collect());
                    self.push(Value::String(ref_val));
                },
                Instruction::FromUnit => {
                    let (val, unit) = (self.pop().num_equiv(), self.pop());
                    match &unit {
                        Value::String(ref_unit) => {
                            let str_unit = self.gc.deref(*ref_unit);
                            match convert_unit_to_si(val, str_unit) {
                                Some(num) => {
                                    self.push(Value::Num(num));
                                },
                                None => {
                                    self.push(Value::Num(f64::NAN));
                                },
                            }
                        }
                        _ => {
                            self.push(Value::Num(f64::NAN));
                        }
                    }
                },
                Instruction::ToUnit => {
                    let (val, unit) = (self.pop().num_equiv(), self.pop());
                    match &unit {
                        Value::String(ref_unit) => {
                            let str_unit = self.gc.deref(*ref_unit);
                            match convert_si_to_unit(val, str_unit) {
                                Some(num) => {
                                    self.push(Value::Num(num));
                                },
                                None => {
                                    self.push(Value::Num(f64::NAN));
                                },
                            }
                        }
                        _ => {
                            self.push(Value::Num(f64::NAN));
                        }
                    }
                },
                Instruction::Acosh => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::acosh(val)));
                },
                Instruction::Sinh  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::sinh(val)));
                },
                Instruction::Asin  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::asin(val)));
                },
                Instruction::Asinh => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::asinh(val)));
                },
                Instruction::Cosh  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::cosh(val)));
                },
                Instruction::Tanh  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::tanh(val)));
                },
                Instruction::Atan  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::atan(val)));
                },
                Instruction::Atanh => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::atanh(val)));
                },
                Instruction::Atan2 => {
                    let (b, a) = (self.pop().num_equiv(), self.pop().num_equiv());
                    self.push(Value::Num(f64::atan2(a, b)));
                },
                Instruction::Log2  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::log2(val)));
                },
                Instruction::Log10 => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::log10(val)));
                },
                Instruction::Ln1p  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::ln_1p(val)));
                },
                Instruction::Ln    => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::ln(val)));
                },
                Instruction::Exp   => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::exp(val)));
                },
                Instruction::Expm1 => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::exp_m1(val)));
                },
                Instruction::Sqrt  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::sqrt(val)));
                },
                Instruction::Cbrt  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::cbrt(val)));
                },
                Instruction::Round => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::round(val)));
                },
                Instruction::Trunc => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::trunc(val)));
                },
                Instruction::Sign  => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(f64::signum(val)));
                },
                Instruction::Fround => {
                    let val = self.pop().num_equiv();
                    self.push(Value::Num(val as f32 as f64));
                },
                Instruction::Random => {
                    let val: f64 = self.rng.gen();
                    self.push(Value::Num(val));
                },
            }
        }
    }
}

