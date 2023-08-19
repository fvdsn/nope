
#[derive(PartialEq, Debug)]
enum TokenValue {
    LeftSqBrkt,
    RightSqBrkt,
    Colon,
    Dot,
    //Number(f64, String),
    String(String),
    Name(String),
}

#[derive(PartialEq, Debug)]
struct Token {
    value: TokenValue,
    line: usize,
    col: usize,
}

#[derive(PartialEq, Debug)]
struct Tokenizer {
    line: usize,
    col: usize,
    index: usize,
    source: String,
    chars: Vec<char>,
    tokens: Vec<Token>,
    done: bool,
}

fn is_wp(c:char) -> bool {
    return  c == ' ' || c == '\t' || c == '\n'
}

fn is_namechar(c:char) -> bool {
    return !is_wp(c) && c != '.' && c != ':' && c != '[' && c != ']'
}

impl Tokenizer {
    fn new(source: String) -> Tokenizer {
        return Tokenizer {
            done: false,
            line: 1,
            col: 1,
            index: 0,
            source: source.to_owned(),
            chars: source.chars().collect(),
            tokens: Vec::new(),
        };
    }
    fn eof(&mut self) -> bool {
        return self.index >= self.chars.len();
    }
    fn cur(&self) -> char {
        return self.chars[self.index];
    }
    fn next(&mut self) {
        if self.eof() {
            return
        }
        let c = self.cur();
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.index += 1;
    }
 
   fn push_token(&mut self, value:TokenValue) {
        self.tokens.push(Token {
            line: self.line,
            col: self.col,
            value: value,
        });
    }
    fn eat_wp(&mut self) {

        loop {
            if self.eof() || !is_wp(self.cur()) {
                return
            }
            self.next();
        }   
    }
    fn tokenize(&mut self) {
        loop {
            self.eat_wp();
            if self.eof() {
                self.done = true;
                return;
            }
            let cur = self.cur();
            if cur == '[' {
                self.push_token(TokenValue::LeftSqBrkt);
                self.next();    
            } else if cur == ']' {
                self.push_token(TokenValue::RightSqBrkt);
                self.next();
            } else if cur == ':' {
                self.push_token(TokenValue::Colon);
                self.next();
            } else if cur == '.' {
                self.push_token(TokenValue::Dot);
                self.next();
            } else if cur == '-' {
                let mut str: Vec<char> = vec![];
                let line = self.line;
                let col = self.col;
                loop {
                    self.next();
                    if self.eof() || is_wp(self.cur()) {
                        break;
                    }
                    str.push(self.cur());
                }
                self.tokens.push(Token {
                    line: line,
                    col: col,
                    value: TokenValue::String(str.iter().collect()),
                });
            } else if cur == '"' || cur == '\'' {
                let mut escape = false;
                let line = self.line;
                let col = self.col;
                let mut str: Vec<char> = vec![];
                let delim = cur;
                loop {
                    self.next();
                    if self.eof() {
                        break;
                    } else if !escape && self.cur() == delim {
                        self.next();
                        break;
                    } else if !escape && self.cur() == '\\' {
                        escape = true;
                        continue
                    } else if escape {
                        if self.cur() ==  'n' {
                            str.push('\n');
                        } else if self.cur() == 't'{
                            str.push('\t');
                        } else {
                            str.push(self.cur());
                        }
                    } else {
                        str.push(self.cur());
                    }
                }
                self.tokens.push(Token {
                    line: line,
                    col: col,
                    value: TokenValue::String(str.iter().collect()),
                });
            } else if is_namechar(cur) {
                let mut name: Vec<char> = vec![];
                let line = self.line;
                let col = self.col;
                loop {
                    if self.eof() {
                        break
                    }
                    let c = self.cur();
                    if is_namechar(c) {
                        name.push(c);
                        self.next();
                    } else {
                        break
                    }
                }
                self.tokens.push(Token {
                    line: line,
                    col: col,
                    value: TokenValue::Name(name.iter().collect()),
                });
            } else {
                self.next();
            }
        }
    }
}


fn main() {
    println!("Hello, world!");
    let mut program = Tokenizer::new(String::from("print 42"));
    program.tokenize();
        
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_empty() {
        let mut program = Tokenizer::new(String::from(""));
        program.tokenize();
        assert_eq!(program.tokens, vec![]);
    }
    
    #[test]
    fn test_parse_lb() {
        let mut program = Tokenizer::new(String::from("["));
        program.tokenize();
        assert_eq!(program.tokens, vec![Token{line:1, col:1, value: TokenValue::LeftSqBrkt}]);
    }

    #[test]
    fn test_parse_wp_lb() {
        let mut program = Tokenizer::new(String::from("  ["));
        program.tokenize();
        assert_eq!(program.tokens, vec![Token{line:1, col:3, value: TokenValue::LeftSqBrkt}]);
    }
    
    #[test]
    fn test_parse_wp_lblb() {
        let mut program = Tokenizer::new(String::from("  [["));
        program.tokenize();
        assert_eq!(
            program.tokens, 
            vec![
                Token{line:1, col:3, value: TokenValue::LeftSqBrkt},
                Token{line:1, col:4, value: TokenValue::LeftSqBrkt},
            ]
        );
    }

    #[test]
    fn test_parse_wp_lblb_nl_rbrb() {
        let mut program = Tokenizer::new(String::from("  [[\n]]  "));
        program.tokenize();
        assert_eq!(
            program.tokens, 
            vec![
                Token{line:1, col:3, value: TokenValue::LeftSqBrkt},
                Token{line:1, col:4, value: TokenValue::LeftSqBrkt},
                Token{line:2, col:1, value: TokenValue::RightSqBrkt},
                Token{line:2, col:2, value: TokenValue::RightSqBrkt},
            ]
        );
    }

    #[test]
    fn test_parse_name() {
        let mut program = Tokenizer::new(String::from("name"));
        program.tokenize();
        assert_eq!(
            program.tokens,
             vec![
                Token{line:1, col:1, value: TokenValue::Name(String::from("name"))},
            ],
        );
    }

    #[test]
    fn test_parse_foo_bar() {
        let mut program = Tokenizer::new(String::from("foo bar"));
        program.tokenize();
        assert_eq!(
            program.tokens,
             vec![
                Token{line:1, col:1, value: TokenValue::Name(String::from("foo"))},
                Token{line:1, col:5, value: TokenValue::Name(String::from("bar"))},
            ],
        );
    }
    
    #[test]
    fn test_parse_foo_name_punct() {
        let mut program = Tokenizer::new(String::from("[foo.bar key:[]]"));
        program.tokenize();
        assert_eq!(
            program.tokens,
             vec![
                Token{line:1, col:1, value: TokenValue::LeftSqBrkt},
                Token{line:1, col:2, value: TokenValue::Name(String::from("foo"))},
                Token{line:1, col:5, value: TokenValue::Dot},
                Token{line:1, col:6, value: TokenValue::Name(String::from("bar"))},
                Token{line:1, col:10, value: TokenValue::Name(String::from("key"))},
                Token{line:1, col:13, value: TokenValue::Colon},
                Token{line:1, col:14, value: TokenValue::LeftSqBrkt},
                Token{line:1, col:15, value: TokenValue::RightSqBrkt},
                Token{line:1, col:16, value: TokenValue::RightSqBrkt},
            ],
        );
    }
    
    #[test]
    fn test_parse_string_foo_bar() {
        let mut program = Tokenizer::new(String::from("-foo -bar-foo"));
        program.tokenize();
        assert_eq!(
            program.tokens,
             vec![
                Token{line:1, col:1, value: TokenValue::String(String::from("foo"))},
                Token{line:1, col:6, value: TokenValue::String(String::from("bar-foo"))},
            ],
        );
    }

    
    #[test]
    fn test_parse_string_quoted() {
        let mut program = Tokenizer::new(String::from("'foo' \"bar'\""));
        program.tokenize();
        assert_eq!(
            program.tokens,
             vec![
                Token{line:1, col:1, value: TokenValue::String(String::from("foo"))},
                Token{line:1, col:7, value: TokenValue::String(String::from("bar'"))},
            ],
        );
    }
}