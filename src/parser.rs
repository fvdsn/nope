
use crate::tokenizer::Tokenizer;
use crate::tokenizer::Token;
use crate::tokenizer::TokenValue;
use crate::tokenizer::TokenizerState;

fn is_reserved_keyword(name: &String) -> bool {
    return name == "true" ||  name == "false" || name == "null" ||
        name == "void" || name == "_" || name == "let" ||
        name == "if" || name == "ife" || name == "do" ||
        name == "end";
}

#[derive(PartialEq, Debug, Clone)]
struct FunctionArg {
    name: String,
    is_func: bool,
    func_arity: usize,
}

#[derive(PartialEq, Debug, Clone)]
enum AstNode {
    // first usize is index of related token in tokens array
    Number(usize, f64),
    String(usize, String),
    Boolean(usize, bool),
    Null(usize),
    Void(usize),
    Error(usize, usize), // second usize is index of value expression
    KeyValue(usize, String, usize), // String is  the key, last usize index of the value expression
    Array(usize, Vec<usize>), // vec of indexes to other ast nodes in the ast array
    Let(usize, String, usize, usize), // String is name of var,
                                     // second usize index of the value expression,
                                     // last usize the expression in which the variable is defined
    Do(usize, usize, usize), // do $expr1 $expr1
    If(usize, usize, usize), // if $cond $expr1 
    IfElse(usize, usize, usize, usize), // ife $cond $expr1 $expr2
    ValueReference(usize, String),    // reference to the variable 'String' that contains a value
    FunctionCall(usize, String, Vec<usize>),    // function call to function named 'String'
    FunctionDef(usize, Vec<FunctionArg>, usize) // last usize is ref to function expression 
}

#[derive(PartialEq, Debug, Clone)]
struct EnvEntry {
    name: String,
    is_func: bool,
    func_args: Vec<FunctionArg>,
}

#[derive(PartialEq, Debug)]
enum ParserState{
    Wip,
    Done,
    Error,
}

#[derive(PartialEq, Debug)]
pub struct ParserError {
    line: usize,
    col: usize,
    message: String,
}

#[derive(PartialEq, Debug)]
pub struct Parser {
    pub tokenizer: Tokenizer,
    ast: Vec<AstNode>,
    env: Vec<EnvEntry>,
    nextindex: usize,
    index: usize,
    state: ParserState,
    errors: Vec<ParserError>,
}

impl Parser {
    pub fn new(source: String) -> Parser {
        return Parser{
            tokenizer: Tokenizer::new(source),
            ast: vec![],
            env: vec![],
            nextindex: 0,
            index: 0,
            state: ParserState::Wip,
            errors: vec![],
        };
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
        let ref node = &self.ast[index];
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
            AstNode::Error(_, expr_index) => {
                println!("{}!", " ".repeat(original_indent));
                self._pretty_print_ast(*expr_index, indent + 2, false);
            },
            AstNode::Array(_, values) => {
                println!("{}[", " ".repeat(original_indent));
                for index in values {
                    self._pretty_print_ast(*index, indent + 2, false);
                }
                println!("{}]", " ".repeat(indent));
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

        }
    }

    fn _pretty_print_error_line(&self, line:usize, col:usize, message: &String) {
        let lines: Vec<&str> = self.tokenizer.source.lines().collect();
        let lineidx = line - 1;
        if lineidx >= 1 {
            println!("{}", lines[lineidx-1]);
        }
        println!("{}", lines[lineidx]);
        let colidx = col -1;
        let mut i:usize = 0;
        loop {
            if i == colidx {
                println!("^");
                break;
            } else {
                print!("-");
                i = i + 1;
            }
        }
        println!("line: {}, col: {} // {}", line, col, message);
        //println!("  - {}", message);
    }

    pub fn pretty_print(&self) {
        let ref tokenizer_state = self.tokenizer.state;
        match tokenizer_state {
            TokenizerState::Error(message) => {
                self._pretty_print_error_line(self.tokenizer.line, self.tokenizer.col, message);
                return;
            },
            _ => {},
        };
        if self.state == ParserState::Error {
            for error in &self.errors {
                self._pretty_print_error_line(error.line, error.col, &error.message);
            }
            return;
        }
        if self.ast.len() > 0 {
            self._pretty_print_ast(self.cur_ast_node_index(), 0, false);
        }
    }

    fn nextt(&mut self) -> &Token {
        assert!(self.tokenizer.tokens.len() > 0);
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
        assert!(tokens.len() > 0);
        if self.nextindex >= self.tokenizer.tokens.len() {
            return &tokens[tokens.len()-1]; // should return Eof
        } else {
            return &tokens[self.nextindex];
        }
    }

    fn peek2t(&self) -> &Token {
        let tokens = &self.tokenizer.tokens;
        assert!(tokens.len() > 0);
        if self.nextindex + 1 >= self.tokenizer.tokens.len() {
            return &tokens[tokens.len()-1]; // should return Eof
        } else {
            return &tokens[self.nextindex + 1];
        }
    }

    fn peek_eof(&self) -> bool {
        let ref token = self.peekt();
        match token {
            Token {value: TokenValue::Eof, ..} => {
                return true;
            },
            _ => {
                return false;
            }
        }
    }

    fn peek_closing_element(&self) -> bool {
        let ref token = self.peekt();
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

    fn peek_rsqbrkt(&self) -> bool {
        let ref token = self.peekt();
        return matches!(token.value, TokenValue::RightSqBrkt);
    }

    fn peek_rightp(&self) -> bool {
        let ref token = self.peekt();
        return matches!(token.value, TokenValue::RightP);
    }

    fn peek_colon(&self) -> bool {
        let ref token = self.peekt();
        return matches!(token.value, TokenValue::Colon);
    }

    fn peek2_colon(&self) -> bool {
        let ref token = self.peek2t();
        return matches!(token.value, TokenValue::Colon);
    }

    fn cur_line_col(&self) ->  (usize, usize){
        let ref token = self.tokenizer.tokens[self.index];
        return (token.line, token.col);
    }

    fn peek_line_col(&self) ->  (usize, usize){
        let ref token = self.peekt();
        return (token.line, token.col);
    }

    fn push_error(&mut self, line: usize, col: usize, message: String) {
        self.state = ParserState::Error;
        self.errors.push(
            ParserError { line: line, col: col, message: message }
        );
    }

    fn push_env_value_entry(&mut self, name: String) {
        self.env.push(EnvEntry { name:name, is_func:false, func_args:vec![] });
    }

    fn push_env_func_entry(&mut self, name: String, args: Vec<FunctionArg>) {
        self.env.push(EnvEntry { name:name, is_func:true, func_args:args });
    }

    fn push_env_arg_func_entry(&mut self, name: String, argc:usize) {
        let mut func_args: Vec<FunctionArg> = vec![];
        for i in 0..argc {
            func_args.push(FunctionArg {
                name: String::from(format!("arg{}",i+1)),
                is_func: false,
                func_arity: 0,
            });
        }
        self.env.push(EnvEntry { name:name, is_func:true, func_args:func_args});
    }

    fn pop_env_entry(&mut self) {
        self.env.pop();
    }

    fn get_env_entry(&self, name: &String) -> Option<EnvEntry> {
        if self.env.len() == 0 {
            return None;
        }
        let mut i = self.env.len() - 1;
        loop {
            let ref entry = self.env[i];
            if entry.name == *name {
                let _entry: EnvEntry = entry.clone();
                return Some(_entry);
            }
            if i > 0 {
                i -= 1;
            } else {
                break;
            }
        }
        return None;
    }
    
    fn cur_ast_node_index(&self) -> usize {
        if self.ast.len() == 0 {
            panic!("should not happen");
        } else {
            return self.ast.len() - 1;
        }
    }

    fn cur_ast_node(&self) -> &AstNode{
        if self.ast.len() == 0 {
            panic!("should not happen");
        } else {
            return &self.ast[self.ast.len() - 1];
        }
    }

    fn finished_parsing(&self) -> bool {
        return self.state != ParserState::Wip;
    }

    fn parse_function(&mut self) {
        let mut func_args:Vec<FunctionArg> = vec![];
        let (fline, fcol) = self.cur_line_col();
        let func_token_index = self.index;
        loop {
            if self.finished_parsing() {
                return;
            } else if self.peek_closing_element() {
                let (line, col) = self.peek_line_col();
                self.push_error(fline, fcol, "start of unterminated function".to_owned());
                self.push_error(line, col, "ERROR: missing | to close the function argument list".to_owned());
                return;
            }
            let ref argname_token = self.nextt().clone();
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
                        if self.finished_parsing() {
                            return;
                        } else if self.peek_closing_element() {
                            let (line, col) = self.peek_line_col();
                            self.push_error(line, col, "ERROR: missing function argument argcount".to_owned());
                            return;
                        }
                        let ref argcount_token = self.nextt().clone();
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
                        is_func:is_func,
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

        if self.finished_parsing() {
            return;
        } else if self.peek_closing_element() {
            let (line, col) = self.peek_line_col();
            self.push_error(line, col, "ERROR: missing function body".to_owned());
            return;
        }
        for arg in &func_args {
            if arg.is_func {
                self.push_env_arg_func_entry(arg.name.clone(), arg.func_arity);
            } else {
                self.push_env_value_entry(arg.name.clone());
            }
        }
        self.parse_expression();
        for _ in &func_args {
            self.pop_env_entry();
        }
        if self.finished_parsing() {
            return;
        }
        self.ast.push(AstNode::FunctionDef(func_token_index, func_args, self.cur_ast_node_index()));
    }

    fn parse_array(&mut self) {
        let mut value_node_indexes:Vec<usize> = vec![];
        let (aline, acol) = self.cur_line_col();
        loop {
            if self.finished_parsing() {
                return;
            } else if self.peek_eof() || self.peek_rightp() {
                let (line, col) = self.peek_line_col();
                self.push_error(aline, acol, "start of unfinished array".to_owned());
                self.push_error(line, col, "ERROR: unfinished array".to_owned());
                return;
            } else if self.peek_rsqbrkt() {
                self.nextt();
                self.ast.push(AstNode::Array(self.index, value_node_indexes));
                return;
            } else {
                if self.peek2_colon() {
                    self.nextt();
                    let keytoken_index = self.index;
                    let keytoken = &self.tokenizer.tokens[self.index];
                    let keystr:String;
                    match keytoken {
                        Token {value: TokenValue::String(ref string, ..), ..} => {
                            keystr = string.to_owned();
                        },
                        Token {value: TokenValue::Name(ref string, ..), ..} => {
                            keystr = string.to_owned();
                        },
                        Token {value: TokenValue::Number(num, ..), ..} => {
                            keystr = num.to_string();
                        },
                        _ => {
                            let (line, col) = self.cur_line_col();
                            self.push_error(aline, acol, "array with invalid key".to_owned());
                            self.push_error(line, col, "ERROR: invalid key definition".to_owned());
                            return;
                        },
                    }
                    self.nextt();
                    self.parse_expression();
                    if self.finished_parsing() {
                        return;
                    }
                    self.ast.push(AstNode::KeyValue(keytoken_index, keystr, self.cur_ast_node_index()));
                    value_node_indexes.push(self.cur_ast_node_index())
                } else {
                    self.parse_expression();
                    value_node_indexes.push(self.cur_ast_node_index()) // put the index of the last parsed astnode
                }
            }
        }
    }

    fn parse_if(&mut self) {
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected condition after 'if'".to_owned());
            return;
        }
        if self.finished_parsing() {
            return;
        }
        let if_idx = self.index;
        self.parse_expression();
        if self.finished_parsing() {
            return;
        }
        let cond_idx = self.cur_ast_node_index();
        if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_error(line, col, "this if is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected expression for 'if'".to_owned());
            return;
        }
        self.parse_expression();
        if self.finished_parsing() {
            return;
        }
        let expr_idx = self.cur_ast_node_index();
        self.ast.push(AstNode::If(if_idx, cond_idx, expr_idx));
    }

    fn parse_do(&mut self) {
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected expression after 'do'".to_owned());
            return;
        }
        if self.finished_parsing() {
            return;
        }
        let do_idx = self.index;
        self.parse_expression();
        if self.finished_parsing() {
            return;
        }
        let expr1_idx = self.cur_ast_node_index();
        if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_error(line, col, "this do is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected expression for 'do'".to_owned());
            return;
        }
        if self.finished_parsing() {
            return;
        }
        self.parse_expression();
        let expr2_idx = self.cur_ast_node_index();
        self.ast.push(AstNode::Do(do_idx, expr1_idx, expr2_idx));
    }

    fn parse_ife(&mut self) {
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected condition after 'ife'".to_owned());
            return;
        }
        if self.finished_parsing() {
            return;
        }
        let if_idx = self.index;
        self.parse_expression();
        if self.finished_parsing() {
            return;
        }
        let cond_idx = self.cur_ast_node_index();
        if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_error(line, col, "this if is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected success expression for 'if'".to_owned());
            return;
        }
        self.parse_expression();
        if self.finished_parsing() {
            return;
        }
        let expr_idx = self.cur_ast_node_index();
        if self.peek_closing_element() {
            let (eline, ecol) = self.peek_line_col();
            self.push_error(line, col, "this if is missing an expression".to_owned());
            self.push_error(eline, ecol, "ERROR: expected else expression for 'if'".to_owned());
            return;
        }
        self.parse_expression();
        if self.finished_parsing() {
            return;
        }
        let expr2_idx = self.cur_ast_node_index();
        self.ast.push(AstNode::IfElse(if_idx, cond_idx, expr_idx, expr2_idx));
    }

    fn parse_let(&mut self) {
        let (line, col) = self.peek_line_col();
        if self.peek_closing_element() {
            self.push_error(line, col, "ERROR: expected identifier after 'let'".to_owned());
            return;
        }
        
        if self.finished_parsing() {
            return;
        }
        let let_idx = self.index;
        
        let ref token = self.nextt().clone();
        match token {
            Token {value: TokenValue::Name(ref string, ..), ..} => {
                if is_reserved_keyword(string) {
                    self.push_error(line, col, "ERROR: cannot redefine reserved keyword".to_owned());
                } else {
                    if self.peek_closing_element() {
                        let (vline, vcol) = self.peek_line_col();
                        self.push_error(line, col, "this variable definition doesn't have a value".to_owned());
                        self.push_error(vline, vcol, "ERROR: expected value for the defined variable".to_owned());
                        return;
                    }
                    self.parse_expression();
                    if self.finished_parsing() {
                        return;
                    }
                    let def_idx = self.cur_ast_node_index();
                    if self.peek_closing_element() {
                        let (vline, vcol) = self.peek_line_col();
                        self.push_error(line, col, "this variable definition doesn't have an expression".to_owned());
                        self.push_error(vline, vcol, "ERROR: expected expression in which to use the defined variable".to_owned());
                        return;
                    }

                    let ref value_node = self.cur_ast_node();

                    match value_node {
                        AstNode::FunctionDef(_, args,_) => {
                            self.push_env_func_entry(string.clone(), args.clone());
                        }
                        _ => {
                            self.push_env_value_entry(string.clone());
                        }
                    };

                    self.parse_expression();
                    self.pop_env_entry();

                    if self.finished_parsing() {
                        return;
                    }
                    let expr_idx = self.cur_ast_node_index();
                    self.ast.push(AstNode::Let(let_idx, string.to_owned(), def_idx, expr_idx));
                }
            },
            _ => {
                self.push_error(line, col, "invalid variable name".to_owned());
            }
        };
    }

    fn parse_name(&mut self, name:String) {
        let (line, col) = self.cur_line_col();
        match self.get_env_entry(&name) {
            Some(env_entry) => {
                if !env_entry.is_func {
                    self.ast.push(AstNode::ValueReference(self.index, name));
                } else {
                    let mut arg_node_indexes: Vec<usize> = vec![];
                    for arg in env_entry.func_args {
                        if self.peek_closing_element() {
                            let (vline, vcol) = self.peek_line_col();
                            self.push_error(line, col, "this function call is missing an argument".to_owned());
                            self.push_error(vline, vcol, "ERROR: expected argument for function call".to_owned());
                            return;
                        }

                        let (aline, acol) = self.peek_line_col();
                        self.parse_expression();

                        if self.finished_parsing() {
                            return;
                        }

                        // type check function arguments
                        if arg.is_func {
                            let func = self.ast[self.ast.len()-1].clone();
                            match func {
                                AstNode::FunctionDef(_, args, _) => {
                                    if args.len() != arg.func_arity {
                                        self.push_error(line, col, "this function call has an argument type error".to_owned());
                                        self.push_error(
                                            aline, acol,
                                            String::from(format!(
                                                "ERROR: expected a function with {} arguments instead of {}",
                                                arg.func_arity, args.len()
                                            ))
                                        );
                                        return;
                                    }
                                },
                                _ => {
                                    self.push_error(line, col, "this function call has an argument type error".to_owned());
                                    self.push_error(
                                        aline, acol,
                                        String::from(format!(
                                            "ERROR: expected a function with {} arguments", arg.func_arity
                                        ))
                                    );
                                    return;
                                }
                            };
                        }
                        arg_node_indexes.push(self.cur_ast_node_index()); 
                    }
                    self.ast.push(AstNode::FunctionCall(self.index, name, arg_node_indexes));
                }
            },
            None => {
                let (line, col) = self.cur_line_col();
                self.push_error(line, col, "ERROR: referenced variable has not been declared".to_owned());
            }
        }
    }

    fn parse_expression(&mut self) {
        let ref token = self.nextt();
        match token {
            Token {value: TokenValue::String(ref string, ..), ..} => {
                let _string = string.to_owned();
                self.ast.push(AstNode::String(self.index, _string));
            },
            Token {value: TokenValue::Number(num, unit), ..} => {
                let mut _num = num.to_owned();
                let _unit = unit.to_owned();
                match _unit.as_deref() {
                    None => {},
                    Some("GT") => {_num *= 1000000000000.0},
                    Some("MT") => {_num *= 1000000000.0},
                    Some("kT") => {_num *= 1000000.0},
                    Some("T") => {_num *= 1000.0},
                    Some("kg") => {},
                    Some("g") => {_num *= 0.001},
                    Some("mg") => {_num *= 0.000001},
                    Some("ug") => {_num *= 0.000000001},
                    Some("ng") => {_num *= 0.000000000001},
                    Some("Ti") => {_num *= 1024.0 * 1024.0 * 1024.0 * 1024.0},
                    Some("Gi") => {_num *= 1024.0 * 1024.0 * 1024.0},
                    Some("Mi") => {_num *= 1024.0 * 1024.0},
                    Some("ki") => {_num *= 1024.0},
                    Some("d") => {_num *= 60.0 * 60.0 * 24.0},
                    Some("h") => {_num *= 60.0 * 60.0},
                    Some("min") => {_num *= 60.0},
                    Some("s") => {},
                    Some("ms") => {_num *= 0.001},
                    Some("us") => {_num *= 0.000001},
                    Some("ns") => {_num *= 0.000000001},
                    Some("deg") => {_num *= std::f64::consts::PI / 180.0},
                    Some("rad") => {},
                    Some("in") => {_num *= 0.024},
                    Some("km") => {_num *= 1000.0},
                    Some("m") => {},
                    Some("dm") => {_num *= 0.1},
                    Some("cm") => {_num *= 0.01},
                    Some("mm") => {_num *= 0.001},
                    Some("um") => {_num *= 0.000001},
                    Some("nm") => {_num *= 0.000000001},
                    Some("lb") => {_num *= 0.453592},
                    Some("oz") => {_num *= 0.0283495},
                    Some("mile") => {_num *= 1609.34},
                    Some("miles") => {_num *= 1609.34},
                    Some("ft") => {_num *= 0.3048},
                    Some("yd") => {_num *= 0.9144},
                    Some("F") => {_num = (_num - 32.0) * 5.0 / 9.0},
                    _ => {
                        let (line, col) = self.cur_line_col();
                        self.push_error(line, col, "ERROR: unknown unit".to_owned());
                        return;
                    }
                }
                self.ast.push(AstNode::Number(self.index, _num));
            },
            Token {value: TokenValue::Name(ref string, ..), ..} => {
                if string == "true" {
                    self.ast.push(AstNode::Boolean(self.index, true));
                } else if string == "false" {
                    self.ast.push(AstNode::Boolean(self.index, false));
                } else if string == "null" {
                    self.ast.push(AstNode::Null(self.index));
                } else if string == "void" || string == "_" || string == "end" {
                    self.ast.push(AstNode::Void(self.index));
                } else if string == "let" {
                    self.parse_let();
                } else if string == "if" {
                    self.parse_if();
                } else if string == "ife" {
                    self.parse_ife();
                } else if string == "do" {
                    self.parse_do();
                } else {
                    let _str:String = string.to_owned();
                    self.parse_name(_str);
                }
            },
            Token {value: TokenValue::Bang, ..} => {
                let bang_index = self.index;
                self.parse_expression();
                if self.finished_parsing() {
                    return;
                }
                self.ast.push(AstNode::Error(bang_index, self.cur_ast_node_index()));
            },
            Token {value: TokenValue::LeftP, ..} => {
                let (pline, pcol) = self.cur_line_col();
                if self.peek_rightp() {
                    self.ast.push(AstNode::Void(self.index));
                    self.nextt();
                    return;
                }
                self.parse_expression();
                if self.finished_parsing() {
                    return;
                }
                if !self.peek_rightp() {
                    self.push_error(pline, pcol, "unclosed '('".to_owned());
                    let (line, col) = self.peek_line_col();
                    self.push_error(line, col, "ERROR: expected closing ')'".to_owned());
                } else {
                    self.nextt();
                }
            },
            Token {value: TokenValue::LeftSqBrkt, ..} => {
                self.parse_array();
            },
            Token {value: TokenValue::Pipe, ..} => {
                self.parse_function();
            },
            Token {value: TokenValue::Eof, ..} => {
                let (line, col) = self.cur_line_col();
                self.push_error(line, col, "ERROR: unexpected end of file".to_owned());
            },
            _ => {
                let (line, col) = self.cur_line_col();
                self.push_error(line, col, "ERROR: unexpected token".to_owned());
            }
        }
    }

    fn set_stdlib(&mut self) {
        // |a f:1|
        self.env.push(
            EnvEntry{ name: "iter".to_owned(), is_func: true, 
                func_args: vec![
                    FunctionArg{is_func: false, func_arity:0, name:"array".to_owned()},
                    FunctionArg{is_func: true,  func_arity:1, name:"iterator".to_owned()},
                ]
            }
        );

        // |f:2 a|
        for name in vec!["map", "filter"] {
            self.env.push(
                EnvEntry{ name: name.to_owned(), is_func: true, 
                    func_args: vec![
                        FunctionArg{is_func: true,  func_arity:2, name:"iterator".to_owned()},
                        FunctionArg{is_func: false, func_arity:0, name:"array".to_owned()},
                    ]
                }
            );
        }

        // |a|
        for name in vec![
            "print", "range", "not", "decr", "incr", "increase",
            "sin", "cos", "tan", "inv", "/max", "/min", "/and",
            "/or", "/eq", "/add", "/mult", "len", "neg"
        ] {
            self.env.push(
                EnvEntry{ name: name.to_owned(), is_func: true, 
                    func_args: vec![
                        FunctionArg{is_func: false, func_arity:0, name:"arg".to_owned()},
                    ]
                }
            );
        }
        // |a b|
        for name in vec![
            "and", "or", "eq", "neq", "add", "sub", "mod", "mult",
            "div", "exp", "<", ">", "<=", ">=", "==", "max", "min"
        ] {
            self.env.push(
                EnvEntry{ name: name.to_owned(), is_func: true, 
                    func_args: vec![
                        FunctionArg{is_func: false, func_arity:0, name:"arg1".to_owned()},
                        FunctionArg{is_func: false, func_arity:0, name:"arg2".to_owned()},
                    ]
                }
            );
        }
    }

    pub fn parse(&mut self) {
        self.tokenizer.tokenize();

        if self.tokenizer.state != TokenizerState::Done {
            return;
        }

        self.set_stdlib();

        loop {
            if self.finished_parsing() {
                break;
            } else if self.peek_eof() {
                self.state = ParserState::Done;
                break;
            } else {
                self.parse_expression();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_empty() {
        let mut parser = Parser::new(String::from(""));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num() {
        let mut parser = Parser::new(String::from("3.1415"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(0, 3.1415)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num_comment() {
        let mut parser = Parser::new(String::from("3.1415#comment"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(0, 3.1415)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num_unit() {
        let mut parser = Parser::new(String::from("1.5mm"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(0, 0.0015)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num_bad_unit() {
        let mut parser = Parser::new(String::from("1.5foo"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_string() {
        let mut parser = Parser::new(String::from("'hello'"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::String(0, "hello".to_owned())
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_true() {
        let mut parser = Parser::new(String::from("true"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Boolean(0, true)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_false() {
        let mut parser = Parser::new(String::from("false"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Boolean(0, false)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_null() {
        let mut parser = Parser::new(String::from("null"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Null(0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_void() {
        let mut parser = Parser::new(String::from("void"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Void(0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_void_lodash() {
        let mut parser = Parser::new(String::from("_"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Void(0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_undeclared_var() {
        let mut parser = Parser::new(String::from("foobar"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_empty_array() {
        let mut parser = Parser::new(String::from("[]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Array(1, vec![])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_num_array() {
        let mut parser = Parser::new(String::from("[3.14]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.14),
            AstNode::Array(2, vec![0])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_nums_array() {
        let mut parser = Parser::new(String::from("[1 2 3]"));
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
        let mut parser = Parser::new(String::from("[true false void null 10 3.14 'hello' -world]"));
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
        let mut parser = Parser::new(String::from("[[]]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Array(2, vec![]),
            AstNode::Array(3, vec![0])
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_unfinished_array() {
        let mut parser = Parser::new(String::from("[[]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Array(2, vec![]),
        ]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_keyval() {
        let mut parser = Parser::new(String::from("[key:99]"));
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
        let mut parser = Parser::new(String::from("['key':99]"));
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
        let mut parser = Parser::new(String::from("[-key:99]"));
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
        let mut parser = Parser::new(String::from("[40.1:99]"));
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
        let mut parser = Parser::new(String::from("[40:99]"));
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
        let mut parser = Parser::new(String::from("[null:99]"));
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
        let mut parser = Parser::new(String::from("[key:]"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_eof_in_keyval() {
        let mut parser = Parser::new(String::from("[key:"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_invalid_key_in_keyval() {
        let mut parser = Parser::new(String::from("[!:88]"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_nested_key_array() {
        let mut parser = Parser::new(String::from("[foo:[bim:99]]"));
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
    fn test_parse_error() {
        let mut parser = Parser::new(String::from("!3.1415"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.1415),
            AstNode::Error(0, 0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_paren() {
        let mut parser = Parser::new(String::from("(3.1415)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.1415),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_paren_unclosed() {
        let mut parser = Parser::new(String::from("(3.1415"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(1, 3.1415),
        ]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_paren_void() {
        let mut parser = Parser::new(String::from("()"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Void(0),
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_basic_let() {
        let mut parser = Parser::new(String::from("let x 3 _"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::Void(3),
            AstNode::Let(0, "x".to_owned(), 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_defines_var() {
        let mut parser = Parser::new(String::from("let x 3 x"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::ValueReference(3, "x".to_owned()),
            AstNode::Let(0, "x".to_owned(), 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_wrong_var() {
        let mut parser = Parser::new(String::from("let x 3 y"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
        ]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_missing_expr() {
        let mut parser = Parser::new(String::from("let x 3"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
        ]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_missing_val() {
        let mut parser = Parser::new(String::from("let x"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_missing_everything() {
        let mut parser = Parser::new(String::from("let"));
        parser.parse();
        assert_eq!(parser.ast, vec![]);
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_redefine_keyword() {
        for kw in ["null", "true", "false", "void", "do", "if", "ife", "end"] {
            let mut parser = Parser::new(String::from(format!("let {} 3 _", kw)));
            parser.parse();
            assert_eq!(parser.ast, vec![]);
            assert_eq!(parser.state, ParserState::Error);
        }
    }

    #[test]
    fn test_parse_let_not_a_varname() {
        for kw in ["3.14", "()", "[]", "|a|", "'str'", "-str"] {
            let mut parser = Parser::new(String::from(format!("let {} 3 _", kw)));
            parser.parse();
            assert_eq!(parser.ast, vec![]);
            assert_eq!(parser.state, ParserState::Error);
        }
    }

    #[test]
    fn test_parse_chained_let() {
        let mut parser = Parser::new(String::from("let x 3 let y 4 [x y]"));
        parser.parse();
        assert_eq!(parser.ast, vec![
            AstNode::Number(2, 3.0),
            AstNode::Number(5, 4.0),
            AstNode::ValueReference(7, "x".to_owned()),
            AstNode::ValueReference(8, "y".to_owned()),
            AstNode::Array(9, vec![2, 3]),
            AstNode::Let(3, "y".to_owned(), 1, 4),
            AstNode::Let(0, "x".to_owned(), 0, 5)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_chained_let_wrong_var() {
        let mut parser = Parser::new(String::from("let x 3 let y 4 [x z]"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_diff_good_scope() {
        let mut parser = Parser::new(String::from("let x [let y 3 [y]] x"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_diff_wrong_scope() {
        let mut parser = Parser::new(String::from("let x [let y 3 [x y]] x"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_diff_wrong_scope2() {
        let mut parser = Parser::new(String::from("let x [let y 3 [y]] y"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_std_func() {
        let mut parser = Parser::new(String::from("(print add 1 2)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_std_func_wrong1() {
        let mut parser = Parser::new(String::from("(print add 1 2 3)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_std_func_wrong2() {
        let mut parser = Parser::new(String::from("(print add 1)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_func_0() {
        let mut parser = Parser::new(String::from("|| 42"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(2, 42.0),
           AstNode::FunctionDef(0, vec![], 0)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_0_missing_body() {
        let mut parser = Parser::new(String::from("||"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_func_1() {
        let mut parser = Parser::new(String::from("|a| a"));
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
            "false", "let", "do", "if", "ife", "'str'", "-str"
        ] {
            let mut parser = Parser::new(String::from(format!("|{}| 3", kw)));
            parser.parse();
            assert_eq!(parser.ast, vec![]);
            assert_eq!(parser.state, ParserState::Error);
        }
    }

    #[test]
    fn test_parse_func_1_missing_body() {
        let mut parser = Parser::new(String::from("|a|"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_func_1_bad_ref() {
        let mut parser = Parser::new(String::from("|a| b"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_func_2() {
        let mut parser = Parser::new(String::from("|a b| [a b]"));
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
        let mut parser = Parser::new(String::from("|a b| [a z]"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_let_func_scope() {
        let mut parser = Parser::new(String::from("let f |a| a f 3"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_let_func_scope_bad() {
        let mut parser = Parser::new(String::from("let f |a| a f a"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[ignore] // FIXME
    #[test]
    fn test_parse_let_func_recursive() {
        let mut parser = Parser::new(String::from("let f |a| f 3 _"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_func_1_funcarg() {
        let mut parser = Parser::new(String::from("(|a:2| a 3 4)"));
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
        let mut parser = Parser::new(String::from("iter [1 2] |v| print v"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_typecheck_func_fail() {
        let mut parser = Parser::new(String::from("iter [1 2] |k v| print v"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_typecheck_func_fail2() {
        let mut parser = Parser::new(String::from("iter [1 2] || print 'hey'"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_ife() {
        let mut parser = Parser::new(String::from("(ife true 99 64)"));
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
        let mut parser = Parser::new(String::from("(ife true 99)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_ife_wrong2() {
        let mut parser = Parser::new(String::from("(ife true)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_ife_wrong3() {
        let mut parser = Parser::new(String::from("(ife)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_if() {
        let mut parser = Parser::new(String::from("(if true 99)"));
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
        let mut parser = Parser::new(String::from("(if true)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_if_wrong2() {
        let mut parser = Parser::new(String::from("(if)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_do() {
        let mut parser = Parser::new(String::from("(do 64 99)"));
        parser.parse();
        assert_eq!(parser.ast, vec![
           AstNode::Number(2, 64.0),
           AstNode::Number(3, 99.0),
           AstNode::Do(1, 0, 1)
        ]);
        assert_eq!(parser.state, ParserState::Done);
    }

    #[test]
    fn test_parse_do_wrong1() {
        let mut parser = Parser::new(String::from("(do 64)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }

    #[test]
    fn test_parse_do_wrong2() {
        let mut parser = Parser::new(String::from("(do)"));
        parser.parse();
        assert_eq!(parser.state, ParserState::Error);
    }
}
