
use crate::tokenizer::Tokenizer;
use crate::tokenizer::Token;
use crate::tokenizer::TokenValue;
use crate::tokenizer::TokenizerState;
use crate::units::convert_unit_to_si;
use crate::config::NopeConfig;
use crate::stdlib::Stdlib;
use crate::penv::{
    FunctionArg,
    Env,
};

use colored::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum UnaryOperator {
    Not,
    Negate,
    Add,
    BitwiseNot,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    AlmostEqual,
    NotAlmostEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
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
}

const MIN_PRECEDENCE: usize = 0;

fn operator_precedence(op: BinaryOperator) -> usize {
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Operator_precedence

    match op {
        BinaryOperator::Equal => 8,
        BinaryOperator::NotEqual => 8,
        BinaryOperator::Less => 9,
        BinaryOperator::LessOrEqual => 9,
        BinaryOperator::Greater => 9,
        BinaryOperator::GreaterOrEqual => 9,
        BinaryOperator::AlmostEqual => 9,
        BinaryOperator::NotAlmostEqual => 9,
        BinaryOperator::Add => 11,
        BinaryOperator::Subtract => 11,
        BinaryOperator::Multiply => 12,
        BinaryOperator::Divide => 12,
        BinaryOperator::Modulo => 12,
        BinaryOperator::Power => 13,

        BinaryOperator::BitwiseAnd => 7,
        BinaryOperator::BitwiseOr => 5,
        BinaryOperator::BitwiseXor => 6,
        BinaryOperator::BitwiseLeftShift => 10,
        BinaryOperator::BitwiseRightShift => 10,
        BinaryOperator::BitwiseZeroRightShift => 10,

        BinaryOperator::I32Add => 11,
        BinaryOperator::I32Subtract => 11,
        BinaryOperator::I32Multiply => 12,
        BinaryOperator::I32Divide => 12,
    }
}

fn operator_associates_right(op: BinaryOperator) -> bool {
    match op {
        BinaryOperator::Power => true,
        _ => false,
    }
}


#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    // first usize is index of related token in tokens array
    Number(usize, f64),
    String(usize, String),
    Boolean(usize, bool),
    Null(usize),
    Void(usize),
    KeyValue(usize, String, usize), // String is  the key, last usize index of the value expression
    Array(usize, Vec<usize>), // vec of indexes to other ast nodes in the ast array
    Let(usize, String, usize, usize), // String is name of var,
                                     // second usize index of the value expression,
                                     // last usize the expression in which the variable is defined
    GlobalLet(usize, String, usize, usize),
    Set(usize, usize, usize), // set $target $expr
    Do(usize, usize, usize), // do $expr1 $expr1
    If(usize, usize, usize), // if $cond $expr1 
    IfElse(usize, usize, usize, usize), // ife $cond $expr1 $expr2
    ValueReference(usize, String),    // reference to the variable 'String' that contains a value
FunctionCall(usize, String, Vec<usize>),    // function call to function named 'String'
    FunctionDef(usize, Vec<FunctionArg>, usize), // last usize is ref to function expression 
    StaticKeyAccess(usize, String, usize),  // string is name of key, last usize is expression of
                                            // which we access the key from
    DynamicKeyAccess(usize, usize, usize), // second usize is the expression that gives the key,
                                            // last usize is the expression that gives the array,
    UnaryOperator(usize, UnaryOperator, usize), 
    BinaryOperator(usize, BinaryOperator, usize, usize), 
    CodeBlock(usize, Vec<usize>),
}

#[derive(PartialEq, Debug)]
enum ParserState{
    Wip,
    Done,
    Error,
    Incomplete,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Severity {
    Info,
    Critical,
}

#[derive(PartialEq, Debug)]
pub struct ParserError {
    line: usize,
    col: usize,
    message: String,
    severity: Severity,
}

#[derive(PartialEq, Debug)]
pub struct Parser {
    config: NopeConfig,
    pub tokenizer: Tokenizer,
    pub ast: Vec<AstNode>,
    pub env: Env,
    block_var_count: usize,
    nextindex: usize,
    index: usize,
    state: ParserState,
    errors: Vec<ParserError>,
}

fn is_reserved_keyword(name: &String) -> bool {
    return name == "true" ||  name == "false" || name == "null" ||
        name == "void" || name == "let" || name == "if" ||
        name == "ife" || name == "do" || name == "end";
}

impl Parser {
    pub fn new_with_env(config: NopeConfig, env: Env, source: String) -> Parser {
        return Parser{
            config,
            env,
            tokenizer: Tokenizer::new(source),
            ast: vec![],
            nextindex: 0,
            block_var_count: 0,
            index: 0,
            state: ParserState::Wip,
            errors: vec![],
        };
    }

    pub fn new(config: NopeConfig, source: String) -> Parser {
        let stdlib = Stdlib::new();

        return Parser::new_with_env(config, stdlib.make_env(), source);
    }

    pub fn print(&self) {
        println!("\nAstNodes:");
        for (i, n) in self.ast.iter().enumerate() {
            println!("  index:{index} node:{node:?}", index=i, node=n);
        }
        println!(
"\n  Parser:
    index: {index},
    state: {state:?}",
            state=self.state,
            index=self.index,
        );
    }

    fn _pretty_print_ast (&self, index:usize, indent:usize, noindent:bool) {
        let original_indent = if noindent {0} else {indent};
        let node = &self.ast[index];
        match node {
            AstNode::Number(_, num) => {
                println!("{}{}", " ".repeat(original_indent), num);
            },
            AstNode::String(_, str) => {
                println!("{}\"{}\"", " ".repeat(original_indent), str);
            },
            AstNode::Boolean(_, val) => {
                println!("{}{}", " ".repeat(original_indent), val);
            },
            AstNode::Null(_) => {
                println!("{}null", " ".repeat(original_indent));
            },
            AstNode::Void(_) => {
                println!("{}_", " ".repeat(original_indent));
            },
            AstNode::ValueReference(_, str) => {
                println!("{}{}", " ".repeat(original_indent), str);
            },
            AstNode::Array(_, values) => {
                println!("{}[", " ".repeat(original_indent));
                for index in values {
                    self._pretty_print_ast(*index, indent + 2, false);
                }
                println!("{}]", " ".repeat(indent));
            },
            AstNode::CodeBlock(_, expressions) => {
                for expression in expressions {
                    self._pretty_print_ast(*expression, indent, false);
                }
            },
            AstNode::KeyValue(_, key, val_index) => {
                print!("{}{}: ", " ".repeat(original_indent), key);
                self._pretty_print_ast(*val_index, indent + 2, true);
            },
            AstNode::Let(_, name, val_index, expr_index) => {
                print!("{}let {} ", " ".repeat(original_indent), name);
                self._pretty_print_ast(*val_index, indent + 2, true);
                self._pretty_print_ast(*expr_index, indent, false);
            },
            AstNode::GlobalLet(_, name, val_index, expr_index) => {
                print!("{}let (global) {} ", " ".repeat(original_indent), name);
                self._pretty_print_ast(*val_index, indent + 2, true);
                self._pretty_print_ast(*expr_index, indent, false);
            },
            AstNode::FunctionCall(_, name, args) => {
                println!("{}{}", " ".repeat(original_indent), name);
                for index in args{
                    self._pretty_print_ast(*index, indent + 2, false);
                }
            },
            AstNode::Do(_, expr_1, expr_2) => {
                println!("{}do", " ".repeat(original_indent));
                self._pretty_print_ast(*expr_1, indent + 2, false);
                self._pretty_print_ast(*expr_2, indent, false);
            },
            AstNode::If(_, cond, expr) => {
                println!("{}if", " ".repeat(original_indent));
                self._pretty_print_ast(*cond, indent + 2, false);
                self._pretty_print_ast(*expr, indent + 2, false);
            },
            AstNode::Set(_, target, expr) => {
                println!("{}set", " ".repeat(original_indent));
                self._pretty_print_ast(*target, indent + 2, false);
                self._pretty_print_ast(*expr, indent + 2, false);
            },
            AstNode::IfElse(_, cond, expr, expr_2) => {
                println!("{}ife", " ".repeat(original_indent));
                self._pretty_print_ast(*cond, indent + 2, false);
                self._pretty_print_ast(*expr, indent + 2, false);
                self._pretty_print_ast(*expr_2, indent, false);
            }
            AstNode::FunctionDef(_, args, expr_body) => {
                print!("{}|", " ".repeat(original_indent));
                for arg in args {
                    if arg.is_func {
                        print!("{}:{} ", arg.name, arg.func_arity);
                    } else {
                        print!("{} ", arg.name);
                    }
                }
                println!("|");
                self._pretty_print_ast(*expr_body, indent + 2, false);
            }
            AstNode::StaticKeyAccess(_, key_name, expr) => {
                println!("{}{}.", " ".repeat(original_indent), key_name);
                self._pretty_print_ast(*expr, indent + 2, false);
            },
            AstNode::DynamicKeyAccess(_, key_expr, expr) => {
                print!("{}[]: ", " ".repeat(original_indent));
                self._pretty_print_ast(*key_expr, indent + 2, true);
                self._pretty_print_ast(*expr, indent + 2, false);
            },
            AstNode::UnaryOperator(_, op, expr) => {
                print!("{}{:?}:", " ".repeat(original_indent), op);
                self._pretty_print_ast(*expr, indent + 2, false);
            },
            AstNode::BinaryOperator(_, op, lexpr, rexpr) => {
                print!("{}{:?}:", " ".repeat(original_indent), op);
                self._pretty_print_ast(*lexpr, indent + 2, false);
                self._pretty_print_ast(*rexpr, indent + 2, false);
            },
        }
    }

    fn _pretty_print_error_line(&self, line:usize, col:usize, severity:Severity, message: &String) {
        let lines: Vec<&str> = self.tokenizer.source.lines().collect();
        let lineidx = line - 1;
        if lineidx >= 1 {
            println!("  {}", lines[lineidx-1].italic());
        }
        println!("  {}", lines[lineidx].italic());
        let colidx = col -1;
        let mut i:usize = 0;
        print!("  ");
        loop {
            if i == colidx {
                match severity {
                    Severity::Critical => {
                        println!("{}", "^".red());
                    },
                    Severity::Info => {
                        println!("{}", "^".blue());
                    },
                };
                break;
            } else {
                print!("-");
                i += 1;
            }
        }
        println!("  line: {}, col: {}   {}", line, col, 
            match severity {
                Severity::Critical => message.red(),
                Severity::Info => message.blue(),
            }
        );
        println!();
    }

    pub fn incomplete(&self) -> bool {
        return self.state == ParserState::Incomplete
    }

    pub fn failed(&self) -> bool {
        return self.tokenizer.failed() || self.parsing_failed();
    }

    pub fn print_errors(&self) {
        println!();
        if let TokenizerState::Error(message) = &self.tokenizer.state {
            self._pretty_print_error_line(self.tokenizer.line, self.tokenizer.col, Severity::Critical, message);
            return;
        }
        if self.parsing_failed() {
            for error in &self.errors {
                self._pretty_print_error_line(error.line, error.col, error.severity, &error.message);
            }
            return;
        }
    }

    pub fn pretty_print(&self) {
        if let TokenizerState::Error(message) = &self.tokenizer.state {
            self._pretty_print_error_line(self.tokenizer.line, self.tokenizer.col, Severity::Critical, message);
            return;
        }
        if self.parsing_failed() {
            for error in &self.errors {
                self._pretty_print_error_line(error.line, error.col, error.severity, &error.message);
            }
            return;
        }
        if !self.ast.is_empty() {
            self._pretty_print_ast(self.cur_ast_node_index(), 0, false);
        }
    }

    fn nextt(&mut self) -> &Token {
        assert!(!self.tokenizer.tokens.is_empty());
        if self.nextindex >= self.tokenizer.tokens.len() {
            return &self.tokenizer.tokens[self.nextindex-1];
        } else {
            self.index = self.nextindex;
            self.nextindex += 1;
            return &self.tokenizer.tokens[self.index];
        }
    }

    fn peekt(&self) -> &Token {
        let tokens = &self.tokenizer.tokens;
        assert!(!tokens.is_empty());
        if self.nextindex >= self.tokenizer.tokens.len() {
            return &tokens[tokens.len()-1]; // should return Eof
        } else {
            return &tokens[self.nextindex];
        }
    }

    fn peek2t(&self) -> &Token {
        let tokens = &self.tokenizer.tokens;
        assert!(!tokens.is_empty());
        if self.nextindex + 1 >= self.tokenizer.tokens.len() {
            return &tokens[tokens.len()-1]; // should return Eof
        } else {
            return &tokens[self.nextindex + 1];
        }
    }

    fn peek_eof(&self) -> bool {
        let token = &self.peekt();
        match token {
            Token {value: TokenValue::Eof, ..} => return true,
            _ => return false,
        }
    }

    fn peek_nleftp(&self) -> bool {
        let token = &self.peekt();
        match token {
            Token {value: TokenValue::NameLeftP, ..} => return true,
            _ => return false,
        }
    }

    fn peek_closing_element(&self) -> bool {
        let token = &self.peekt();
        match token {
            Token {value: TokenValue::Eof, ..} => {
                return true;
            },
            Token {value: TokenValue::RightP, ..} => {
                return true;
            },
            Token {value: TokenValue::RightSqBrkt, ..} => {
                return true;
            },
            _ => {
                return false;
            }
        }
    }

    fn peek_binary_op(&self) -> Option<BinaryOperator> {
        let token = &self.peekt();
        return match token {
            Token {value: TokenValue::Operator(op), ..} => {
                match op.as_str() {
                    "=="   => Some(BinaryOperator::Equal),
                    "!="   => Some(BinaryOperator::NotEqual),
                    "<="   => Some(BinaryOperator::LessOrEqual),
                    ">="   => Some(BinaryOperator::GreaterOrEqual),
                    "+-="  => Some(BinaryOperator::AlmostEqual),
                    "!+-=" => Some(BinaryOperator::NotAlmostEqual),
                    "**"   => Some(BinaryOperator::Power),
                    "<"    => Some(BinaryOperator::Less),
                    ">"    => Some(BinaryOperator::Greater),
                    "+"    => Some(BinaryOperator::Add),
                    "-"    => Some(BinaryOperator::Subtract),
                    "*"    => Some(BinaryOperator::Multiply),
                    "/"    => Some(BinaryOperator::Divide),
                    "%"    => Some(BinaryOperator::Modulo),
                    "~|"   => Some(BinaryOperator::BitwiseOr),
                    "~&"   => Some(BinaryOperator::BitwiseAnd),
                    "~^"   => Some(BinaryOperator::BitwiseXor),
                    "~+"   => Some(BinaryOperator::I32Add),
                    "~-"   => Some(BinaryOperator::I32Subtract),
                    "~*"   => Some(BinaryOperator::I32Multiply),
                    "~/"   => Some(BinaryOperator::I32Divide),
                    "~<<"   => Some(BinaryOperator::BitwiseLeftShift),
                    "~>>"   => Some(BinaryOperator::BitwiseRightShift),
                    "~>>>"   => Some(BinaryOperator::BitwiseZeroRightShift),
                    _ => None, 
                }
            }
            _ => {
                None
            }
        }
    }

    fn peek_rsqbrkt(&self) -> bool {
        let token = &self.peekt();
        return matches!(token.value, TokenValue::RightSqBrkt);
    }

    fn peek_rightp(&self) -> bool {
        let token = &self.peekt();
        return matches!(token.value, TokenValue::RightP);
    }

    fn peek_swp(&self) -> bool {
        let token = &self.peekt();
        return matches!(token.value, TokenValue::Swp);
    }

    fn peek_comma(&self) -> bool {
        let token = &self.peekt();
        return matches!(token.value, TokenValue::Comma);
    }

    fn peek_equal(&self) -> bool {
        let token = &self.peekt();
        return matches!(token.value, TokenValue::Equal);
    }

    fn peek_colon(&self) -> bool {
        let token = &self.peekt();
        return matches!(token.value, TokenValue::Colon);
    }

    fn peek2_colon(&self) -> bool {
        let token = &self.peek2t();
        return matches!(token.value, TokenValue::Colon);
    }

    fn peek2_dot(&self) -> bool {
        let token = &self.peek2t();
        return matches!(token.value, TokenValue::Dot);
    }

    fn cur_line_col(&self) ->  (usize, usize){
        let token = &self.tokenizer.tokens[self.index];
        return (token.line, token.col);
    }

    fn peek_line_col(&self) ->  (usize, usize){
        let token = &self.peekt();
        return (token.line, token.col);
    }

    fn push_info(&mut self, line: usize, col: usize, message: String) {
        self.errors.push(
            ParserError { line, col, message, severity:Severity::Info }
        );
    }

    fn push_error(&mut self, line: usize, col: usize, message: String) {
        self.state = ParserState::Error;
        self.errors.push(
            ParserError { line, col, message, severity:Severity::Critical }
        );
    }

    fn push_incomplete(&mut self, line: usize, col: usize, message: String) {
        self.state = ParserState::Incomplete;
        self.errors.push(
            ParserError { line, col, message, severity:Severity::Critical }
        );
    }

    fn cur_ast_node_index(&self) -> usize {
        if self.ast.is_empty() {
            panic!("should not happen");
        } else {
            return self.ast.len() - 1;
        }
    }

    fn cur_ast_node(&self) -> &AstNode{
        if self.ast.is_empty() {
            panic!("should not happen");
        } else {
            return &self.ast[self.ast.len() - 1];
        }
    }

    pub fn parsing_failed(&self) -> bool {
        return self.state == ParserState::Error || self.state == ParserState::Incomplete;
    }

    fn parse_function_def(&mut self, func_name: Option<&str>) {
        // parses a function definiton |a b:n| body
        // when starting the `|` must have already been consumed
        // - adds all the argument definition to the environment
        // - adds a reference to self as func_name to the environment if func_name is provided
        //    - this allows recursive function calls
        // - parses the body as an expression and puts it on the ast stack
        // - adds the function definition AstNode on top of the stack, referencing the body
        // - drops what was added from the environment
        
        let mut func_args:Vec<FunctionArg> = vec![];
        let (fline, fcol) = self.cur_line_col();
        let func_token_index = self.index;
        loop {
            if self.peek_closing_element() {
                let (line, col) = self.peek_line_col();
                self.push_info(fline, fcol, "start of unterminated function".to_owned());
                self.push_error(line, col,  "ERROR: missing | to close the function argument list".to_owned());
                return;
            }
            let argname_token = &self.nextt().clone();
            let (line, col) = self.cur_line_col();
            match argname_token {
                Token {value: TokenValue::Name(ref name, ..), ..} => {
                    if is_reserved_keyword(name) {
                        self.push_error(line, col, "ERROR: cannot redefine reserved keyword".to_owned());
                        return;
                    }
                    let mut argc: usize = 0;
                    let mut is_func: bool = false;

                    // FIXME handle duplicate arguments names
                    // TODO handle _ dummy arguments

                    if self.peek_colon() { // parsing "|arg:n| to argc / is_func
                                           
                        self.nextt();

                        if self.peek_closing_element() {
                            let (line, col) = self.peek_line_col();
                            self.push_error(line, col, "ERROR: missing function argument argcount".to_owned());
                            return;
                        }
                        let argcount_token = &self.nextt().clone();
                        let (line, col) = self.cur_line_col();
                        match argcount_token { // parsing the 'n' of |arg:n|
                            Token {value: TokenValue::Number(num, unit), ..} => {
                                if !unit.is_none() {
                                    self.push_error(line, col, "ERROR: badly formatted argcount".to_owned());
                                    return;
                                }
                                if num.fract() != 0.0 {
                                    self.push_error(line, col, "ERROR: badly formatted argcount".to_owned());
                                    return;
                                }
                                argc = *num as usize;
                                is_func = true;
                            }
                            _ => {
                                self.push_error(line, col, "ERROR: expected integer here".to_owned());
                                return;
                            }
                        }
                    }

                    func_args.push(FunctionArg {
                        name:name.to_owned(),
                        is_func,
                        func_arity:argc,
                    });
                },
                Token {value: TokenValue::Pipe, ..} => {
                    break;
                }
                _ => {
                    self.push_error(line, col, "ERROR: invalid function argument".to_owned());
                    return;
                }

            }
        }

        if self.peek_closing_element() {
            let (line, col) = self.peek_line_col();
            self.push_error(line, col, "ERROR: missing function body".to_owned());
            return;
        }

        // if we have a name for the function, push a reference to it in the environment
        // to allow recursion
        if let Some(name) = func_name {
            self.env.push_func_entry(name.to_string(), func_args.clone());
        }

        // create an environment entry for each function argument
        for arg in &func_args {
            if arg.is_func {
                self.env.push_arg_func_entry(arg.name.clone(), arg.func_arity);
            } else {
                self.env.push_value_entry(arg.name.clone());
            }
        }

        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }

        for _ in &func_args {
            self.env.pop_entry();
        }

        if func_name.is_some() {
            self.env.pop_entry();
        }

        self.ast.push(AstNode::FunctionDef(func_token_index, func_args, self.cur_ast_node_index()));
    }

    fn parse_array_or_dynamic_key_access(&mut self) {
        // parses [a b k:c ...]
        //  - when starting, the `[` token must have already been consumed
        //  - puts all the inside expressions as AstNodes on the stack
        //  - puts an Array AstNode on top referencing the sub expression nodes
        let mut value_node_indexes:Vec<usize> = vec![];
        let (aline, acol) = self.cur_line_col();
        let array_token_index = self.index;
        let mut has_func_val = false;
        let mut fline = 0;
        let mut fcol = 0;
        loop {
            if self.peek_eof() || self.peek_rightp() {
                let (line, col) = self.peek_line_col();
                self.push_info(aline, acol, "start of unfinished array".to_owned());
                self.push_error(line, col, "ERROR: unfinished array".to_owned());
                return;
            } else if self.peek_rsqbrkt() {
                self.nextt();
                if self.peek_swp() || self.peek_rsqbrkt() || self.peek_rightp() || self.peek_eof() {
                    if self.peek_swp() {
                        self.nextt();
                    }
                    if has_func_val {
                        self.push_error(fline, fcol, "ERROR functions are not allowed in data structures".to_owned());
                    }
                    self.ast.push(AstNode::Array(self.index, value_node_indexes));
                    return;
                } else if value_node_indexes.len() != 1 {
                    self.push_error(aline, acol, "ERROR key index must have exactly one key".to_owned());
                } else {
                    let key_node_index = self.cur_ast_node_index();
                    
                    let ast_node = self.cur_ast_node();

                    if let AstNode::FunctionDef(_, args, _) = ast_node {
                        if args.len() != 2 {
                            self.push_error(aline, acol, "ERROR filter function must have 2 arguments (key, value)".to_owned());
                        }
                    }

                    self.parse_expression(false, false, None);
                    if self.parsing_failed() {
                        return;
                    }
                    let value_node_index = self.cur_ast_node_index();
                    self.ast.push(AstNode::DynamicKeyAccess(array_token_index, key_node_index, value_node_index));
                    return;
                }
            } else if self.peek2_colon() {
                self.nextt();
                let keytoken_index = self.index;
                let keytoken = &self.tokenizer.tokens[self.index];
                let keystr:String = match keytoken {
                    Token {value: TokenValue::String(ref string, ..), ..} => {
                        string.to_owned()
                    },
                    Token {value: TokenValue::Name(ref string, ..), ..} => {
                        string.to_owned()
                    },
                    Token {value: TokenValue::Number(num, ..), ..} => {
                        num.to_string()
                    },
                    _ => {
                        let (line, col) = self.cur_line_col();
                        self.push_info(aline, acol, "array with invalid key".to_owned());
                        self.push_error(line, col, "ERROR: invalid key definition".to_owned());
                        return;
                    },
                };
                self.nextt();
                self.parse_expression(false, false, None);
                if self.parsing_failed() {
                    return;
                }
                self.ast.push(AstNode::KeyValue(keytoken_index, keystr, self.cur_ast_node_index()));
                value_node_indexes.push(self.cur_ast_node_index())
            } else {
                let (line, col) = self.peek_line_col();

                self.parse_expression(false, false, None);
                if self.parsing_failed() {
                    return;
                }

                let ast_node = self.cur_ast_node();

                if let AstNode::FunctionDef(_, _, _) = ast_node {
                    if !has_func_val {
                        has_func_val = true;
                        fline = line;
                        fcol = col;
                    }
                }
                value_node_indexes.push(self.cur_ast_node_index()) // put the index of the last parsed astnode
            }
        }
    }

    fn parse_if(&mut self) {
        // parses if cond expr
        // - if must have already been consumed
        // - parses and puts the cond & expr AstNodes on the ast stack
        // - adds a If AstNode referencing the cond and the expr on top of the ast stack
        
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected condition after 'if'".to_owned());
            return;
        }

        let if_idx = self.index;
        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }
        let cond_idx = self.cur_ast_node_index();
        if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_info(line, col, "this if is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected expression for 'if'".to_owned());
            return;
        }
        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }
        let expr_idx = self.cur_ast_node_index();
        self.ast.push(AstNode::If(if_idx, cond_idx, expr_idx));
    }

    fn parse_set(&mut self) {
        // parses set target expr
        // - set must have already been consumed
        
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected target after 'set'".to_owned());
            return;
        }

        let set_idx = self.index;
        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }

        let target_idx = self.cur_ast_node_index();
        if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_info(line, col, "this set is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected expression for 'set'".to_owned());
            return;
        }
        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }
        let expr_idx = self.cur_ast_node_index();
        self.ast.push(AstNode::Set(set_idx, target_idx, expr_idx));
    }

    fn parse_do(&mut self) {
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected expression after 'do'".to_owned());
            return;
        }

        let do_idx = self.index;
        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }
        let expr1_idx = self.cur_ast_node_index();

        if self.peek_rightp() {
            self.ast.push(AstNode::Void(do_idx));
        } else if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_info(line, col, "this do is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected expression for 'do'".to_owned());
            return;
        } else {
            self.parse_expression(false, false, None);
            if self.parsing_failed() {
                return;
            }
        }
        let expr2_idx = self.cur_ast_node_index();
        self.ast.push(AstNode::Do(do_idx, expr1_idx, expr2_idx));
    }

    fn parse_ife(&mut self) {
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected condition after 'ife'".to_owned());
            return;
        }

        let if_idx = self.index;
        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }
        let cond_idx = self.cur_ast_node_index();
        if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_info(line, col, "this if is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected success expression for 'if'".to_owned());
            return;
        }

        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }

        let expr_idx = self.cur_ast_node_index();
        if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_info(line, col, "this if is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected else expression for 'if'".to_owned());
            return;
        }

        self.parse_expression(false, false, None);
        if self.parsing_failed() {
            return;
        }

        let expr2_idx = self.cur_ast_node_index();
        self.ast.push(AstNode::IfElse(if_idx, cond_idx, expr_idx, expr2_idx));
    }

    fn parse_let(&mut self, global_scope: bool, code_block: bool) {
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected identifier after 'let'".to_owned());
            return;
        }
        
        let let_idx = self.index;

        let token = &self.nextt().clone();
        match token {
            Token {value: TokenValue::Name(ref var_name, ..), ..} => {
                if is_reserved_keyword(var_name) {
                    self.push_error(line, col, "ERROR: cannot redefine reserved keyword".to_owned());
                } else {
                    if self.peek_closing_element() {
                        let (vline, vcol) = self.peek_line_col();
                        self.push_info(line, col, "this variable definition doesn't have a value".to_owned());
                        self.push_error(vline, vcol, "ERROR: expected value for the defined variable".to_owned());
                        return;
                    }

                    if self.peek_equal() { // we accept an optional '='; "let x = 42" or "let x 42"
                        self.nextt();
                    }
        
                    self.parse_expression(false, false, Some(var_name));
                    if self.parsing_failed() {
                        return;
                    }
                    let def_idx = self.cur_ast_node_index();


                    let value_node = &self.cur_ast_node();

                    match value_node {
                        AstNode::FunctionDef(_, args,_) => {
                            self.env.push_func_entry(var_name.clone(), args.clone());
                        }
                        _ => {
                            self.env.push_value_entry(var_name.clone());
                        }
                    };

                    if !self.peek_closing_element() {
                        self.parse_expression(global_scope, code_block, None);
                        if self.parsing_failed() {
                            return;
                        }
                    } else {
                        self.ast.push(AstNode::Void(let_idx));
                    }

                    if code_block {
                        self.block_var_count += 1;
                    } else if !global_scope {
                        self.env.pop_entry();
                    }

                    let expr_idx = self.cur_ast_node_index();
                    if global_scope {
                        self.ast.push(AstNode::GlobalLet(let_idx, var_name.to_owned(), def_idx, expr_idx));
                    } else {
                        self.ast.push(AstNode::Let(let_idx, var_name.to_owned(), def_idx, expr_idx));
                    }
                }
            },
            _ => {
                self.push_error(line, col, "invalid variable name".to_owned());
            }
        };
    }

    fn parse_func_call(&mut self, name:String) {
        let (line, col) = self.cur_line_col();
        let mut uses_commas = false;
        let mut explicit_func_call = false;

        if self.peek_nleftp() { // function call is of the form 'foo(...)' instead of 'foo ...'
            explicit_func_call = true;
            uses_commas = true;
            self.nextt();
        }

        match self.env.get_entry(&name) {
            Some(env_entry) => {
                if !env_entry.is_func {
                    if explicit_func_call {
                        let (line, col) = self.cur_line_col();
                        self.push_error(line, col, "ERROR: the referenced variable is not a function".to_owned());
                        return;
                    }
                    self.ast.push(AstNode::ValueReference(self.index, name));
                } else {
                    let mut arg_node_indexes: Vec<usize> = vec![];
                    for (arg_index, arg) in env_entry.func_args.iter().enumerate() {
                        if self.peek_eof() {
                            let (vline, vcol) = self.peek_line_col();
                            self.push_info(line, col, "this function call is missing an argument".to_owned());
                            self.push_incomplete(vline, vcol, "ERROR: expected argument for function call".to_owned());
                            return;
                        } else if self.peek_closing_element() {
                            let (vline, vcol) = self.peek_line_col();
                            self.push_info(line, col, "this function call is missing an argument".to_owned());
                            self.push_error(vline, vcol, "ERROR: expected argument for function call".to_owned());
                            return;
                        }

                        let (aline, acol) = self.peek_line_col();

                        if explicit_func_call {
                            self.parse_expression(false, false, None);
                        } else {
                            self.parse_unary(false, false, None);
                        }

                        if self.parsing_failed() {
                            return;
                        }

                        // type check function arguments
                        if arg.is_func {
                            let func = self.ast[self.ast.len()-1].clone();
                            match func {
                                AstNode::FunctionDef(_, args, _) => {
                                    if args.len() != arg.func_arity {
                                        self.push_info(line, col, "this function call has an argument type error".to_owned());
                                        self.push_error(
                                            aline, acol,
                                            format!(
                                                "ERROR: expected a function with {} arguments instead of {}",
                                                arg.func_arity, args.len()
                                            )
                                        );
                                        return;
                                    }
                                },
                                _ => {
                                    self.push_info(line, col, "this function call has an argument type error".to_owned());
                                    self.push_error(
                                        aline, acol,
                                        format!(
                                            "ERROR: expected a function with {} arguments", arg.func_arity
                                        )
                                    );
                                    return;
                                }
                            };
                        }

                        if arg_index < env_entry.func_args.len() - 1 {
                            if self.peek_comma() {
                                if arg_index == 0 {
                                    uses_commas = true;
                                    self.nextt();
                                } else if uses_commas {
                                    self.nextt();
                                } else {
                                    self.push_error(line, col, "ERROR: this function call is missing some commas between its arguments".to_owned());
                                    return;
                                }
                            } else if uses_commas {
                                let (cline, ccol) = self.peek_line_col();
                                self.push_info(line, col, "this function call is missing some commas between its arguments".to_owned());
                                self.push_error(cline, ccol, "ERROR: expected a comma here".to_owned());
                                return;
                            }
                        }

                        arg_node_indexes.push(self.cur_ast_node_index()); 
                    }
                    self.ast.push(AstNode::FunctionCall(self.index, name, arg_node_indexes));
                    
                    if explicit_func_call {
                        if self.peek_rightp() {
                            self.nextt();
                        } else if self.peek_eof() {
                            let (vline, vcol) = self.peek_line_col();
                            self.push_info(line, col, "this function call is missing a closing parenthesis".to_owned());
                            self.push_incomplete(vline, vcol, "ERROR: expected ')'".to_owned());
                            return;
                        } else {
                            let (vline, vcol) = self.peek_line_col();
                            self.push_info(line, col, "this function call is missing a closing parenthesis".to_owned());
                            self.push_error(vline, vcol, "ERROR: expected expected ')'".to_owned());
                            return;
                        }
                    }
                }
            },
            None => {
                let (line, col) = self.cur_line_col();
                self.push_error(line, col, "ERROR: undeclared variable".to_owned());
            }
        }
    }

    fn parse_static_key_access(&mut self, key_name: String) {
        // parses foo.expr
        // - foo must have already be consumed, is passed as key_name
        // - the dot after foo must have already been peeked
        // - parses expr and adds it on the ast
        // - adds a StaticKeyAccess AstNode on top of the ast, referencing the expr

        let key_name_idx = self.index;

        let _dot = &self.nextt();

        let (dline, dcol) = self.cur_line_col();  // line col of the dot

        if self.peek_eof() {
            self.push_incomplete(dline, dcol, "ERROR: expected expression after key access".to_owned());
        } else if self.peek_closing_element() {
            self.push_error(dline, dcol, "ERROR: expected expression after key access".to_owned());
            return;
        }

        self.parse_expression(false, false, None);

        if self.parsing_failed() {
            return;
        }

        self.ast.push(AstNode::StaticKeyAccess(key_name_idx, key_name, self.cur_ast_node_index()));
    }

    fn parse_expression(&mut self, global_scope: bool, code_block: bool, var_name: Option<&str>) {
        // - global_scope is true if the expression makes variables declaration part of the global
        // scope
        // - code_block is true if the expression makes variables declarations part of a multi
        // expression code block
        // - var_name is the name of the variable this expression will be assigned to;
        // used to handle recursion

        self.parse_unary(global_scope, code_block, var_name);

        if self.parsing_failed() {
            return;
        }
        
        let left_node_index = self.cur_ast_node_index();

        self.parse_binary(left_node_index, MIN_PRECEDENCE, var_name);
    }

    fn parse_binary(
        &mut self,
        mut left_node_index: usize,
        min_precedence: usize,
        var_name: Option<&str>
    ) {
        // https://en.wikipedia.org/wiki/Operator-precedence_parser

        loop {
            if let Some(op) = self.peek_binary_op() {
                if operator_precedence(op) < min_precedence {
                    return;
                }

                self.nextt();

                let op_token_index = self.index;

                self.parse_unary(false, false, var_name);

                let mut right_node_index = self.cur_ast_node_index();

                if self.parsing_failed() {
                    return;
                }

                loop {
                    if let Some(op_ahead) = self.peek_binary_op() {
                        if operator_associates_right(op_ahead) {
                            if operator_precedence(op_ahead) < operator_precedence(op) {
                                break;
                            }
                        } else {
                            if operator_precedence(op_ahead) <= operator_precedence(op) {
                                break;
                            }
                        }

                        let precedence_increment = if operator_precedence(op_ahead) == operator_precedence(op) {
                            0
                        } else {
                            1
                        };

                        self.parse_binary(
                            right_node_index,
                            operator_precedence(op) + precedence_increment,
                            var_name,
                        );
                        
                        if self.parsing_failed() {
                            return;
                        }

                        right_node_index = self.cur_ast_node_index();
                    } else {
                        break;
                    }
                }

                self.ast.push(AstNode::BinaryOperator(op_token_index, op, left_node_index, right_node_index));

                left_node_index = self.cur_ast_node_index();
            } else {
                break;
            }
        }
    }

    fn parse_unary(&mut self, global_scope: bool, code_block: bool, var_name: Option<&str>) {

        let dot_after_token = self.peek2_dot();
        let token = &self.nextt();
        match token {
            Token {value: TokenValue::String(ref string, ..), ..} => {
                let _string = string.to_owned();
                self.ast.push(AstNode::String(self.index, _string));
            },
            Token {value: TokenValue::Number(num, None), ..} => {
                let _num = num.to_owned();
                self.ast.push(AstNode::Number(self.index, _num));
            },
            Token {value: TokenValue::Number(num, Some(unit)), ..} => {
                let _num = convert_unit_to_si(*num, unit);
                match _num {
                    Some(__num) => {
                        self.ast.push(AstNode::Number(self.index, __num));
                    }
                    None => {
                        let (line, col) = self.cur_line_col();
                        self.push_error(line, col, "ERROR: unknown unit".to_owned());
                    }
                };
            },
            Token {value: TokenValue::PipeLeft, ..} => {
                self.parse_expression(false, false, None);
            },
            Token {value: TokenValue::Operator(ref operator), ..} => {
                if operator == "!" || operator == "-"  || operator == "+" {

                    let op = if operator == "!" { 
                        UnaryOperator::Not 
                    } else if operator == "-" {
                        UnaryOperator::Negate 
                    } else if operator == "~!" {
                        UnaryOperator::BitwiseNot
                    } else if operator == "+" { 
                        UnaryOperator::Add 
                    } else {
                        panic!("unknown operator");
                    };

                    let op_token_index = self.index;

                    self.parse_expression(false, false, None);
                    if self.parsing_failed() {
                        return;
                    }
                    self.ast.push(AstNode::UnaryOperator(
                        op_token_index,
                        op,
                        self.cur_ast_node_index(),
                    ));
                } else {
                    let (line, col) = self.cur_line_col();
                    self.push_error(line, col, "ERROR: unexpected operator".to_owned());
                }
            },
            Token {value: TokenValue::Name(ref name, ..), ..} => {
                if dot_after_token {
                    let key_name:String = name.to_owned();
                    self.parse_static_key_access(key_name);
                } else if name == "true" {
                    self.ast.push(AstNode::Boolean(self.index, true));
                } else if name == "false" {
                    self.ast.push(AstNode::Boolean(self.index, false));
                } else if name == "null" {
                    self.ast.push(AstNode::Null(self.index));
                } else if name == "void" || name == "_" || name == "end" {
                    self.ast.push(AstNode::Void(self.index));
                } else if name == "let" {
                    self.parse_let(global_scope, code_block);
                } else if name == "set" {
                    self.parse_set();
                } else if name == "if" {
                    self.parse_if();
                } else if name == "ife" {
                    self.parse_ife();
                } else if name == "do" {
                    self.parse_do();
                } else {
                    let func_name:String = name.to_owned();
                    self.parse_func_call(func_name);
                }
            },
            Token {value: TokenValue::LeftP, ..} => {
                let (pline, pcol) = self.cur_line_col();
                if self.peek_rightp() {
                    self.ast.push(AstNode::Void(self.index));
                    self.nextt();
                    return;
                }
                self.parse_expression(false, false, var_name);
                if self.parsing_failed() {
                    return;
                }
                if self.peek_eof() {
                    self.push_info(pline, pcol, "unclosed '('".to_owned());
                    let (line, col) = self.peek_line_col();
                    self.push_incomplete(line, col, "ERROR: expected closing ')'".to_owned());
                } else if !self.peek_rightp() {
                    self.push_info(pline, pcol, "unclosed '('".to_owned());
                    let (line, col) = self.peek_line_col();
                    self.push_error(line, col, "ERROR: expected closing ')'".to_owned());
                } else {
                    self.nextt();
                }
            },
            Token {value: TokenValue::LeftSqBrkt, ..} => {
                self.parse_array_or_dynamic_key_access();
            },
            Token {value: TokenValue::Pipe, ..} => {
                self.parse_function_def(var_name);
            },
            Token {value: TokenValue::Eof, ..} => {
                let (line, col) = self.cur_line_col();
                self.push_incomplete(line, col, "ERROR: unexpected end of file".to_owned());
            },
            _ => {
                let (line, col) = self.cur_line_col();
                self.push_error(line, col, "ERROR: unexpected token".to_owned());
            }
        }
    }

    fn parse_code_block(&mut self, global_scope: bool) {
        let mut expressions_indexes:Vec<usize> = vec![];
        let code_block_token_index = self.index;

        let cur_block_var_count = self.block_var_count;

        loop {
            if self.peek_eof() {
                break;
            }

            self.parse_expression(global_scope, true, None);
            if self.parsing_failed() {
                return;
            }

            expressions_indexes.push(self.cur_ast_node_index())
        }

        if !global_scope {
            for _ in cur_block_var_count..self.block_var_count {
                self.env.pop_entry();
            }
            self.block_var_count = cur_block_var_count;
        }


        if expressions_indexes.len() >= 2 {
            self.ast.push(AstNode::CodeBlock(code_block_token_index, expressions_indexes));
        }
    }


    pub fn parse(&mut self) {
        if self.config.debug {
            println!("tokenize...");
        }

        self.tokenizer.tokenize();
        if self.tokenizer.failed() {
            return;
        }

        if self.config.debug {
            println!("build ast...");
        }

        self.parse_code_block(true);

        if !self.parsing_failed() {
            self.state = ParserState::Done;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: NopeConfig = NopeConfig {
        debug: true,
        echo_result: false,
    };
    
    #[test]
    fn test_parse_empty() {
        let mut parser = Parser::new(CONFIG, String::from(""));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num() {
        let mut parser = Parser::new(CONFIG, String::from("3.1415"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(0, 3.1415)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num_comment() {
        let mut parser = Parser::new(CONFIG, String::from("3.1415#comment"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(0, 3.1415)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num_unit() {
        let mut parser = Parser::new(CONFIG, String::from("1.5mm"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(0, 0.0015)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num_bad_unit() {
        let mut parser = Parser::new(CONFIG, String::from("1.5foo"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_string() {
        let mut parser = Parser::new(CONFIG, String::from("'hello'"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::String(0, "hello".to_owned())
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_true() {
        let mut parser = Parser::new(CONFIG, String::from("true"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Boolean(0, true)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_false() {
        let mut parser = Parser::new(CONFIG, String::from("false"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Boolean(0, false)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_null() {
        let mut parser = Parser::new(CONFIG, String::from("null"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Null(0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_void() {
        let mut parser = Parser::new(CONFIG, String::from("void"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Void(0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_void_lodash() {
        let mut parser = Parser::new(CONFIG, String::from("_"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Void(0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_undeclared_var() {
        let mut parser = Parser::new(CONFIG, String::from("foobar"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_empty_array() {
        let mut parser = Parser::new(CONFIG, String::from("[]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Array(1, vec![])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num_array() {
        let mut parser = Parser::new(CONFIG, String::from("[3.14]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.14),
            AstNode::Array(2, vec![0])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_nums_array() {
        let mut parser = Parser::new(CONFIG, String::from("[1 2 3]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 1.0),
            AstNode::Number(2, 2.0),
            AstNode::Number(3, 3.0),
            AstNode::Array(4, vec![0,1,2])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_mixed_array() {
        let mut parser = Parser::new(CONFIG, String::from("[true false void null 10 3.14 'hello' ~world]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Boolean(1, true),
            AstNode::Boolean(2, false),
            AstNode::Void(3),
            AstNode::Null(4),
            AstNode::Number(5, 10.0),
            AstNode::Number(6, 3.14),
            AstNode::String(7, "hello".to_owned()),
            AstNode::String(8, "world".to_owned()),
            AstNode::Array(9, vec![0,1,2,3,4,5,6,7])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_nested_array() {
        let mut parser = Parser::new(CONFIG, String::from("[[]]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Array(2, vec![]),
            AstNode::Array(3, vec![0])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_unfinished_array() {
        let mut parser = Parser::new(CONFIG, String::from("[[]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Array(2, vec![]),
        ]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_keyval() {
        let mut parser = Parser::new(CONFIG, String::from("[key:99]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(3, 99.0),
            AstNode::KeyValue(1, "key".to_owned(), 0),
            AstNode::Array(4, vec![1]),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_keyval_string() {
        let mut parser = Parser::new(CONFIG, String::from("['key':99]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(3, 99.0),
            AstNode::KeyValue(1, "key".to_owned(), 0),
            AstNode::Array(4, vec![1]),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_keyval_dash_string() {
        let mut parser = Parser::new(CONFIG, String::from("[~key:99]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(3, 99.0),
            AstNode::KeyValue(1, "key".to_owned(), 0),
            AstNode::Array(4, vec![1]),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_keyval_num() {
        let mut parser = Parser::new(CONFIG, String::from("[40.1:99]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(3, 99.0),
            AstNode::KeyValue(1, "40.1".to_owned(), 0),
            AstNode::Array(4, vec![1]),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_keyval_int() {
        let mut parser = Parser::new(CONFIG, String::from("[40:99]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(3, 99.0),
            AstNode::KeyValue(1, "40".to_owned(), 0),
            AstNode::Array(4, vec![1]),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_keyval_kw() {
        let mut parser = Parser::new(CONFIG, String::from("[null:99]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(3, 99.0),
            AstNode::KeyValue(1, "null".to_owned(), 0),
            AstNode::Array(4, vec![1]),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_missing_val_in_keyval() {
        let mut parser = Parser::new(CONFIG, String::from("[key:]"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_eof_in_keyval() {
        let mut parser = Parser::new(CONFIG, String::from("[key:"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Incomplete);
    }

    #[test]
    fn test_parse_invalid_key_in_keyval() {
        let mut parser = Parser::new(CONFIG, String::from("[+:88]"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_nested_key_array() {
        let mut parser = Parser::new(CONFIG, String::from("[foo:[bim:99]]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(6, 99.0),
            AstNode::KeyValue(4, "bim".to_owned(), 0),
            AstNode::Array(7, vec![1]),
            AstNode::KeyValue(1, "foo".to_owned(), 2),
            AstNode::Array(8, vec![3]),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_paren() {
        let mut parser = Parser::new(CONFIG, String::from("(3.1415)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.1415),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_paren_unclosed() {
        let mut parser = Parser::new(CONFIG, String::from("(3.1415"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.1415),
        ]);
        assert_eq!(parser.state, ParserState::Incomplete);
    }

    #[test]
    fn test_parse_paren_unclosed_2() {
        let mut parser = Parser::new(CONFIG, String::from("(add 1 2"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Incomplete);
    }

    #[test]
    fn test_parse_paren_unclosed_3() {
        let mut parser = Parser::new(CONFIG, String::from("(add 1"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Incomplete);
    }

    #[test]
    fn test_parse_paren_void() {
        let mut parser = Parser::new(CONFIG, String::from("()"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Void(0),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_basic_let() {
        let mut parser = Parser::new(CONFIG, String::from("let x 3 _"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::Void(3),
            AstNode::GlobalLet(0, "x".to_owned(), 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_defines_var() {
        let mut parser = Parser::new(CONFIG, String::from("let x 3 x"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::ValueReference(3, "x".to_owned()),
            AstNode::GlobalLet(0, "x".to_owned(), 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_with_equal() {
        let mut parser = Parser::new(CONFIG, String::from("let x = 3 x"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(3, 3.0),
            AstNode::ValueReference(4, "x".to_owned()),
            AstNode::GlobalLet(0, "x".to_owned(), 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_wrong_var() {
        let mut parser = Parser::new(CONFIG, String::from("let x 3 y"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
        ]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_missing_expr() {
        let mut parser = Parser::new(CONFIG, String::from("let x 3"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0), 
            AstNode::Void(0),
            AstNode::GlobalLet(0, "x".to_owned(), 0, 1),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_missing_val() {
        let mut parser = Parser::new(CONFIG, String::from("let x"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_missing_everything() {
        let mut parser = Parser::new(CONFIG, String::from("let"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_redefine_keyword() {
        for kw in ["null", "true", "false", "void", "do", "if", "ife", "end"] {
            let mut parser = Parser::new(CONFIG, String::from(format!("let {} 3 _", kw)));
            parser.parse();
            assert_eq!(parser.ast, vec![]);
            assert_eq!(parser.state, ParserState::Error);
        }
    }

    #[test]
    fn test_parse_let_not_a_varname() {
        for kw in ["3.14", "()", "[]", "|a|", "'str'", "~str"] {
            let mut parser = Parser::new(CONFIG, String::from(format!("let {} 3 _", kw)));
            parser.parse();
            assert_eq!(parser.ast, vec![]);
            assert_eq!(parser.state, ParserState::Error);
        }
    }

    #[test]
    fn test_parse_chained_let() {
        let mut parser = Parser::new(CONFIG, String::from("let x 3 let y 4 [x y]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::Number(5, 4.0),
            AstNode::ValueReference(7, "x".to_owned()),
            AstNode::ValueReference(8, "y".to_owned()),
            AstNode::Array(9, vec![2, 3]),
            AstNode::GlobalLet(3, "y".to_owned(), 1, 4),
            AstNode::GlobalLet(0, "x".to_owned(), 0, 5)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_chained_let_wrong_var() {
        let mut parser = Parser::new(CONFIG, String::from("let x 3 let y 4 [x z]"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_diff_good_scope() {
        let mut parser = Parser::new(CONFIG, String::from("let x [let y 3 [y]] x"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_diff_wrong_scope() {
        let mut parser = Parser::new(CONFIG, String::from("let x [let y 3 [x y]] x"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_diff_wrong_scope2() {
        let mut parser = Parser::new(CONFIG, String::from("let x [let y 3 [y]] y"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_std_func() {
        let mut parser = Parser::new(CONFIG, String::from("(print add 1 2)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_std_func_wrong1() {
        let mut parser = Parser::new(CONFIG, String::from("(print add 1 2 3)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_std_func_wrong2() {
        let mut parser = Parser::new(CONFIG, String::from("(print add 1)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_std_func_wrong2_inc() {
        let mut parser = Parser::new(CONFIG, String::from("print add 1"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Incomplete);
    }

    #[test]
    fn test_parse_func_implicit() {
        let mut parser = Parser::new(CONFIG, String::from("random"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::FunctionCall(0, "random".to_owned(), vec![])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_explicit() {
        let mut parser = Parser::new(CONFIG, String::from("random()"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::FunctionCall(1, "random".to_owned(), vec![]) //FIXME index ?
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_explicit2() {
        let mut parser = Parser::new(CONFIG, String::from("add(1,2)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 1.0),
            AstNode::Number(4, 2.0),
            AstNode::FunctionCall(4, "add".to_owned(), vec![0, 1])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_explicit_no_commas() {
        let mut parser = Parser::new(CONFIG, String::from("add(1 2)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_func_0() {
        let mut parser = Parser::new(CONFIG, String::from("|| 42"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(2, 42.0),
           AstNode::FunctionDef(0, vec![], 0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_0_missing_body() {
        let mut parser = Parser::new(CONFIG, String::from("||"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_func_1() {
        let mut parser = Parser::new(CONFIG, String::from("|a| a"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::ValueReference(3, "a".to_owned()),
           AstNode::FunctionDef(0, vec![
                FunctionArg { name: "a".to_owned(), is_func: false, func_arity: 0 }
           ], 0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_wrong_arg() {
        for kw in [
            "3.14", "()", "[]", "null", "void", "true",
            "false", "let", "do", "if", "ife", "'str'", "~str"
        ] {
            let mut parser = Parser::new(CONFIG, String::from(format!("|{}| 3", kw)));
            parser.parse();
            assert_eq!(parser.ast, vec![]);
            assert_eq!(parser.state, ParserState::Error);
        }
    }

    #[test]
    fn test_parse_func_1_missing_body() {
        let mut parser = Parser::new(CONFIG, String::from("|a|"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_func_1_bad_ref() {
        let mut parser = Parser::new(CONFIG, String::from("|a| b"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_func_2() {
        let mut parser = Parser::new(CONFIG, String::from("|a b| [a b]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::ValueReference(5, "a".to_owned()),
           AstNode::ValueReference(6, "b".to_owned()),
           AstNode::Array(7, vec![0, 1]),
           AstNode::FunctionDef(0, vec![
                FunctionArg { name: "a".to_owned(), is_func: false, func_arity: 0 },
                FunctionArg { name: "b".to_owned(), is_func: false, func_arity: 0 }
           ], 2)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_2_bad_ref() {
        let mut parser = Parser::new(CONFIG, String::from("|a b| [a z]"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_func_scope() {
        let mut parser = Parser::new(CONFIG, String::from("let f |a| a f 3"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_func_scope_bad() {
        let mut parser = Parser::new(CONFIG, String::from("let f |a| a f a"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_func_recursive() {
        let mut parser = Parser::new(CONFIG, String::from("let f |a| f 3 _"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_func_recursive_p() {
        let mut parser = Parser::new(CONFIG, String::from("let f (|a| f 3) _"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_1_funcarg() {
        let mut parser = Parser::new(CONFIG, String::from("(|a:2| a 3 4)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(7, 3.0),
           AstNode::Number(8, 4.0),
           AstNode::FunctionCall(8, "a".to_owned(), vec![0, 1]), 
           AstNode::FunctionDef(1, vec![FunctionArg { name: "a".to_owned(), is_func: true, func_arity: 2 }], 2)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_typecheck_func() {
        let mut parser = Parser::new(CONFIG, String::from("iter [1 2] |v| print v"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_typecheck_func_fail() {
        let mut parser = Parser::new(CONFIG, String::from("iter [1 2] |k v| print v"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_typecheck_func_fail2() {
        let mut parser = Parser::new(CONFIG, String::from("iter [1 2] || print 'hey'"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_ife() {
        let mut parser = Parser::new(CONFIG, String::from("(ife true 99 64)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Boolean(2, true),
           AstNode::Number(3, 99.0),
           AstNode::Number(4, 64.0),
           AstNode::IfElse(1, 0, 1, 2)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_ife_wrong1() {
        let mut parser = Parser::new(CONFIG, String::from("(ife true 99)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_ife_wrong2() {
        let mut parser = Parser::new(CONFIG, String::from("(ife true)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_ife_wrong3() {
        let mut parser = Parser::new(CONFIG, String::from("(ife)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_if() {
        let mut parser = Parser::new(CONFIG, String::from("(if true 99)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Boolean(2, true),
           AstNode::Number(3, 99.0),
           AstNode::If(1, 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_if_wrong1() {
        let mut parser = Parser::new(CONFIG, String::from("(if true)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_if_wrong2() {
        let mut parser = Parser::new(CONFIG, String::from("(if)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_do() {
        let mut parser = Parser::new(CONFIG, String::from("(do 64 99)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(2, 64.0),
           AstNode::Number(3, 99.0),
           AstNode::Do(1, 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_do_early_close() {
        let mut parser = Parser::new(CONFIG, String::from("(do 64)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 64.0),
            AstNode::Void(1),
            AstNode::Do(1, 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_do_wrong1() {
        let mut parser = Parser::new(CONFIG, String::from("[do 64]"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_do_wrong2() {
        let mut parser = Parser::new(CONFIG, String::from("(do)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_do_wrong3() {
        let mut parser = Parser::new(CONFIG, String::from("do 64"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_foo_dot_bar() {
        let mut parser = Parser::new(CONFIG, String::from("foo.'bar'"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::String(2, "bar".to_string()),
           AstNode::StaticKeyAccess(0, "foo".to_string(), 0),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_foo_dot_bim_dot_bar() {
        let mut parser = Parser::new(CONFIG, String::from("foo.bim.'bar'"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::String(4, "bar".to_string()),
           AstNode::StaticKeyAccess(2, "bim".to_string(), 0),
           AstNode::StaticKeyAccess(0, "foo".to_string(), 1),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_foo_dot_add_1_2() {
        let mut parser = Parser::new(CONFIG, String::from("foo.add 1 2"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(3, 1.0),
           AstNode::Number(4, 2.0),
           AstNode::FunctionCall(4, "add".to_string(), vec![0, 1]),
           AstNode::StaticKeyAccess(0, "foo".to_string(), 2)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dangling_key() {
        let mut parser = Parser::new(CONFIG, String::from("(foo.)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_dynkey_num() {
        let mut parser = Parser::new(CONFIG, String::from("[3]12"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(1, 3.0),
           AstNode::Number(3, 12.0),
           AstNode::DynamicKeyAccess(0, 0, 1),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_null() {
        let mut parser = Parser::new(CONFIG, String::from("[null]12"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Null(1),
           AstNode::Number(3, 12.0),
           AstNode::DynamicKeyAccess(0, 0, 1),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_void() {
        let mut parser = Parser::new(CONFIG, String::from("[_]12"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Void(1),
           AstNode::Number(3, 12.0),
           AstNode::DynamicKeyAccess(0, 0, 1),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_bool() {
        let mut parser = Parser::new(CONFIG, String::from("[true]12"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Boolean(1, true),
           AstNode::Number(3, 12.0),
           AstNode::DynamicKeyAccess(0, 0, 1),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_str() {
        let mut parser = Parser::new(CONFIG, String::from("['key']12"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::String(1, "key".to_string()),
           AstNode::Number(3, 12.0),
           AstNode::DynamicKeyAccess(0, 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_var() {
        let mut parser = Parser::new(CONFIG, String::from("let pi 3.14 [pi]12"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(2, 3.14),
           AstNode::ValueReference(4, "pi".to_string()),
           AstNode::Number(6, 12.0),
           AstNode::DynamicKeyAccess(3, 1, 2),
           AstNode::GlobalLet(0, "pi".to_string(), 0, 3)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_key_filter() {
        let mut parser = Parser::new(CONFIG, String::from("[key:45]12"));
        // filters all elements with key 'key' and value 45
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(3, 45.0),
           AstNode::KeyValue(1, "key".to_string(), 0),
           AstNode::Number(5, 12.0),
           AstNode::DynamicKeyAccess(0, 1, 2)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_func_filter() {
        let mut parser = Parser::new(CONFIG, String::from("[|k v| true]12"));
        // filters all elements with key 'key' and value 45
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Boolean(5, true),
            AstNode::FunctionDef(1, vec![
                 FunctionArg { name: "k".to_string(), is_func: false, func_arity: 0 },
                 FunctionArg { name: "v".to_string(), is_func: false, func_arity: 0 }
            ], 0),
            AstNode::Number(7, 12.0),
            AstNode::DynamicKeyAccess(0, 1, 2)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_rec() {
        let mut parser = Parser::new(CONFIG, String::from("[3][4]12"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(1, 3.0),
           AstNode::Number(4, 4.0),
           AstNode::Number(6, 12.0),
           AstNode::DynamicKeyAccess(3, 1, 2),
           AstNode::DynamicKeyAccess(0, 0, 3)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_to_array() {
        let mut parser = Parser::new(CONFIG, String::from("[3][4]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.0),
            AstNode::Number(4, 4.0),
            AstNode::Array(5, vec![1]),
            AstNode::DynamicKeyAccess(0, 0, 2)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_in_array() {
        let mut parser = Parser::new(CONFIG, String::from("[[3][4]]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::Number(5, 4.0),
            AstNode::Array(6, vec![1]),
            AstNode::DynamicKeyAccess(1, 0, 2),
            AstNode::Array(7, vec![3])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_in_array_paren() {
        let mut parser = Parser::new(CONFIG, String::from("[([3])[4]]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(3, 3.0),
            AstNode::Array(4, vec![0]),
            AstNode::Number(7, 4.0),
            AstNode::Array(8, vec![2]),
            AstNode::Array(9, vec![1, 3])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dynkey_func_filter_wrong_argc0() {
        let mut parser = Parser::new(CONFIG, String::from("[||true]33"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_dynkey_func_filter_wrong_argc1() {
        let mut parser = Parser::new(CONFIG, String::from("[|v|true]33"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_dynkey_func_filter_wrong_argc3() {
        let mut parser = Parser::new(CONFIG, String::from("[|a b c|true]33"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_no_func_in_arrays() {
        let mut parser = Parser::new(CONFIG, String::from("[|a|true]"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_dont_redefine_void() {
        // _ must always stay a void value. assigning a value to it has no effect
        let mut parser = Parser::new(CONFIG, String::from("let _ |a| true _"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_dont_redefine_void_in_func_args() {
        // _ must always stay a void value.
        let mut parser = Parser::new(CONFIG, String::from("|_:2| _"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_no_commas() {
        let mut parser = Parser::new(CONFIG, String::from("let foo |a b c| _ foo 1 2 3"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_all_commas() {
        let mut parser = Parser::new(CONFIG, String::from("let foo |a b c| _ foo 1, 2, 3"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_missing_comma1() {
        let mut parser = Parser::new(CONFIG, String::from("let foo |a b c| _ foo 1 2, 3"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_missing_comma2() {
        let mut parser = Parser::new(CONFIG, String::from("let foo |a b c| _ foo 1, 2 3"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_code_block() {
        let mut parser = Parser::new(CONFIG, String::from("print 3.14 print 4.92"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.14),
            AstNode::FunctionCall(1, "print".to_owned(), vec![0]),
            AstNode::Number(3, 4.92),
            AstNode::FunctionCall(3, "print".to_owned(), vec![2]),
            AstNode::CodeBlock(0, vec![1, 3])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_code_block_env_carry_over() {
        let mut parser = Parser::new(CONFIG, String::from("let a 3 _ print a"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::Void(3),
            AstNode::GlobalLet(0, "a".to_owned(), 0, 1),
            AstNode::ValueReference(5, "a".to_owned()),
            AstNode::FunctionCall(5, "print".to_owned(), vec![3]),
            AstNode::CodeBlock(0, vec![2, 4])
        ]);
    }

    #[test]
    fn test_parse_code_block_env_carry_over2() {
        let mut parser = Parser::new(CONFIG, String::from("let a 3 let b 4 _ add a b"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_code_block_env_carry_over_wrong() {
        let mut parser = Parser::new(CONFIG, String::from("if true let a 3 _ print a"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_local_let() {
        let mut parser = Parser::new(CONFIG, String::from("let a || let x 3 33 _"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
        assert_eq!(parser.ast, vec![
           AstNode::Number(6, 3.0),
           AstNode::Number(7, 33.0),
           AstNode::Let(4, "x".to_owned(), 0, 1),
           AstNode::FunctionDef(2, vec![], 2),
           AstNode::Void(8),
           AstNode::GlobalLet(0, "a".to_owned(), 3, 4)
        ]);
    }

    #[test]
    fn test_parse_not() {
        let mut parser = Parser::new(CONFIG, String::from("!32"));
        parser.parse();
        assert_eq!(parser.ast, vec![
              AstNode::Number(1, 32.0),
              AstNode::UnaryOperator(0, UnaryOperator::Not, 0),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_neg() {
        let mut parser = Parser::new(CONFIG, String::from("-32"));
        parser.parse();
        assert_eq!(parser.ast, vec![
              AstNode::Number(1, 32.0),
              AstNode::UnaryOperator(0, UnaryOperator::Negate, 0),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_plus() {
        let mut parser = Parser::new(CONFIG, String::from("+32"));
        parser.parse();
        assert_eq!(parser.ast, vec![
              AstNode::Number(1, 32.0),
              AstNode::UnaryOperator(0, UnaryOperator::Add, 0),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_not_plus_minus() {
        let mut parser = Parser::new(CONFIG, String::from("!+-32"));
        parser.parse();
        assert_eq!(parser.ast, vec![
              AstNode::Number(3, 32.0),
              AstNode::UnaryOperator(2, UnaryOperator::Negate, 0),
              AstNode::UnaryOperator(1, UnaryOperator::Add, 1),
              AstNode::UnaryOperator(0, UnaryOperator::Not, 2),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_binary_add() {
        let mut parser = Parser::new(CONFIG, String::from("1+1"));
        parser.parse();
        assert_eq!(parser.ast, vec![
              AstNode::Number(0, 1.0),
              AstNode::Number(2, 1.0),
              AstNode::BinaryOperator(1, BinaryOperator::Add, 0, 1),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_binary_add_add() {
        let mut parser = Parser::new(CONFIG, String::from("1+1+1"));
        parser.parse();
        assert_eq!(parser.ast, vec![
              AstNode::Number(0, 1.0),
              AstNode::Number(2, 1.0),
              AstNode::BinaryOperator(1, BinaryOperator::Add, 0, 1),
              AstNode::Number(4, 1.0),
              AstNode::BinaryOperator(3, BinaryOperator::Add, 2, 3),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_binary_pow_pow() {
        let mut parser = Parser::new(CONFIG, String::from("1**1**1"));
        parser.parse();
        assert_eq!(parser.ast, vec![
              AstNode::Number(0, 1.0),
              AstNode::Number(2, 1.0),
              AstNode::Number(4, 1.0),
              AstNode::BinaryOperator(3, BinaryOperator::Power, 1, 2),
              AstNode::BinaryOperator(1, BinaryOperator::Power, 0, 3),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_binary_op_precedence() {
        let mut parser = Parser::new(CONFIG, String::from("1+2*3==3*2+1"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(0, 1.0),
            AstNode::Number(2, 2.0),
            AstNode::Number(4, 3.0),
            AstNode::BinaryOperator(3, BinaryOperator::Multiply, 1, 2),
            AstNode::BinaryOperator(1, BinaryOperator::Add, 0, 3),
            AstNode::Number(6, 3.0),
            AstNode::Number(8, 2.0),
            AstNode::BinaryOperator(7, BinaryOperator::Multiply, 5, 6),
            AstNode::Number(10, 1.0),
            AstNode::BinaryOperator(9, BinaryOperator::Add, 7, 8),
            AstNode::BinaryOperator(5, BinaryOperator::Equal, 4, 9),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_no_parenthesis_mixed_with_operators() {
        let mut parser = Parser::new(CONFIG, String::from("neg 3 + neg 5"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.0),
            AstNode::FunctionCall(1, "neg".to_owned(), vec![0]),
            AstNode::Number(4, 5.0),
            AstNode::FunctionCall(4, "neg".to_owned(), vec![2]),
            AstNode::BinaryOperator(2, BinaryOperator::Add, 1, 3)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_no_parenthesis_mixed_with_operators2() {
        let mut parser = Parser::new(CONFIG, String::from("neg(3 + not 5)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::Number(5, 5.0),
            AstNode::FunctionCall(5, "not".to_owned(), vec![1]),
            AstNode::BinaryOperator(3, BinaryOperator::Add, 0, 2),
            AstNode::FunctionCall(5, "neg".to_owned(), vec![3])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }
}
