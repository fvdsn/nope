
use crate::tokenizer::Tokenizer;
use crate::tokenizer::Token;
use crate::tokenizer::TokenValue;

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
    Error(String),
}

#[derive(PartialEq, Debug)]
pub struct Parser {
    pub tokenizer: Tokenizer,
    ast: Vec<AstNode>,
    nextindex: usize,
    index: usize,
    state: ParserState,
}

impl Parser {
    pub fn new(source: String) -> Parser {
        return Parser{
            tokenizer: Tokenizer::new(source),
            ast: vec![],
            nextindex: 0,
            index: 0,
            state: ParserState::Wip,
        };
    }

    pub fn print(&self) {
        println!(
"Parser:
  state: {state:?},
  index: {index}",
            state=self.state,
            index=self.index,
        );
        println!("\nAst:");
        for (i, n) in self.ast.iter().enumerate() {
            println!("  index:{index} node:{node:?}", index=i, node=n);
        }
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

    pub fn pretty_print_ast(&self) {
        println!("\nFormatted Ast:");
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

    fn parse_array(&mut self) {
        let mut value_node_indexes:Vec<usize> = vec![];
        loop {
            if self.state != ParserState::Wip {
                return;
            } else if self.peek_eof() {
                self.state = ParserState::Error("Unexpected Eof while parsing array".to_owned());
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
                            self.state = ParserState::Error("invalid key definition".to_owned());
                            return;
                        },
                    }
                    self.nextt();
                    self.parse_expression();
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
            Token {value: TokenValue::Number(num, ..), ..} => {
                let _num = num.to_owned();
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
                    self.state = ParserState::Error(format!("Unexpected token in expression '{}'", string));
                }
            },
            Token {value: TokenValue::LeftSqBrkt, ..} => {
                self.parse_array()
            },
            _ => {
                self.state = ParserState::Error(format!("Unexpected Token in expression {:?}", token));
            }
        }
    }

    pub fn parse(&mut self) {
        println!("tokens: {}", self.tokenizer.tokens.len());
        self.tokenizer.tokenize();
        println!("tokens: {}", self.tokenizer.tokens.len());
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
