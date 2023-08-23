
use crate::tokenizer::Tokenizer;
use crate::tokenizer::Token;
use crate::tokenizer::TokenValue;
use crate::tokenizer::TokenizerState;

#[derive(PartialEq, Debug)]
enum AstNode {
    // first usize is index of related token in tokens array
    Number(usize, f64),
    String(usize, String),
    Boolean(usize, bool),
    Null(usize),
    Void(usize),
    KeyValue(usize, String, usize), // String is 
    Array(usize, Vec<usize>) // vec of indexes to other ast nodes in the ast array
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
            self._pretty_print_ast(self.ast.len()-1, 0, false);
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

    fn peek_rsqbrkt(&self) -> bool {
        let ref token = self.peekt();
        return matches!(token.value, TokenValue::RightSqBrkt);
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
            ParserError { line: line, col: col, message: message}
        );
    }

    fn parse_array(&mut self) {
        let mut value_node_indexes:Vec<usize> = vec![];
        let (aline, acol) = self.cur_line_col();
        loop {
            if self.state != ParserState::Wip {
                return;
            } else if self.peek_eof() {
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
                    if self.state != ParserState::Wip {
                        return;
                    }
                    self.ast.push(AstNode::KeyValue(keytoken_index, keystr, self.ast.len()-1));
                    value_node_indexes.push(self.ast.len()-1)
                } else {
                    self.parse_expression();
                    value_node_indexes.push(self.ast.len()-1) // put the index of the last parsed astnode
                 }
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
                } else if string == "void" || string == "_" {
                    self.ast.push(AstNode::Void(self.index));
                } else {
                    let (line, col) = self.cur_line_col();
                    self.push_error(line, col, "ERROR: referenced variable has not been declared".to_owned());
                }
            },
            Token {value: TokenValue::LeftSqBrkt, ..} => {
                self.parse_array()
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

    pub fn parse(&mut self) {
        self.tokenizer.tokenize();

        if self.tokenizer.state != TokenizerState::Done {
            return;
        }

        loop {
            if self.state != ParserState::Wip {
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
}
