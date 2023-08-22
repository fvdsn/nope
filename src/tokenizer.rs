#[derive(PartialEq, Debug, Clone)]
pub enum TokenValue {
    LeftSqBrkt,
    RightSqBrkt,
    Colon,
    Dot,
    Pipe,
    Bang,
    Comma,
    Eof,
    Number(f64, Option<String>),
    String(String),
    Name(String),
    Comment(String),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub value: TokenValue,
    pub line: usize,
    pub col: usize,
}


#[derive(PartialEq, Debug)]
pub enum TokenizerState{
    Wip,
    Done,
    Error(String),
}

#[derive(PartialEq, Debug)]
pub struct Tokenizer {
    source: String,
    chars: Vec<char>, // source as vector of characters
    nextindex: usize, // next character to look at, must start at 0
    index: usize, // index of current char in chars, only valid after first call to nextc()
    line: usize, // line of character at 'index', starts at 1
    col: usize,  // collumn of character at 'index', starts at 1
    pub tokens: Vec<Token>, // resulting tokens
    pub state: TokenizerState,
}

fn is_eof(c:char) -> bool {
    return c == '\0';
}

fn is_wp(c:char) -> bool {
    return  c == ' ' || c == '\t' || c == '\n'
}

fn is_separator(c:char) -> bool {
    return c == '.' || c == ':' || c == '[' || c == ']' || c == '!' 
        || c == '|' || c == '"' || c == '\'' || c == '#' || c == ',';
}

fn is_dashstr_separator(c:char) -> bool {
    return c == ':' || c == '[' || c == ']' || c == ',';
}

fn is_num_separator(c:char) -> bool {
    return c == ':' || c == '[' || c == ']' || c == '!' || c == '|' 
        || c == '"' || c == '\'' || c == '-' || c == '#' || c == ',';
}

fn is_namechar(c:char) -> bool {
    return !is_wp(c) && !is_separator(c);
}


fn is_digit(c:char) -> bool {
    return c.is_digit(10);
}

fn is_alpha(c:char) -> bool {
    return c.is_alphabetic();
}

impl Tokenizer {
    pub fn new(source: String) -> Tokenizer {
        return Tokenizer {
            line: 1,
            col: 1,
            index: 0,
            nextindex: 0,
            source: source.to_owned(),
            chars: source.chars().collect(),
            tokens: Vec::new(),
            state: TokenizerState::Wip,
        };
    }

    pub fn print(&self) {
        println!(
"Tokenizer:
  line: {line}
  col: {col}
  state: {state:?}",
            line=self.line,
            col=self.col,
            state=self.state,
        );
        println!("\nTokens:");
        for t in self.tokens.iter() {
            println!("  line:{line} col:{col} value:{val:?}", line=t.line, col=t.col, val=t.value);
        }
    }

    fn nextc(&mut self) -> char {
        if self.nextindex == 0 {
            self.line = 1;
            self.col = 1;
        }

        if self.nextindex >= self.chars.len() {
            return '\0';
        }

        self.index = self.nextindex;

        if self.index >= 1 {
            if self.chars[self.index-1] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }

        self.nextindex += 1;

        return self.chars[self.index];
    }

    fn peek1(&self) -> char {
        if self.nextindex >= self.chars.len() {
            return '\0';
        } else {
            return self.chars[self.nextindex];
        }
    }

    fn match_and_push_token(&mut self, token: &str, value:TokenValue) -> bool {
        if token.len() + self.index > self.chars.len() {
            return false;
        } else {
            for (i, c) in token.chars().enumerate() {
                if c != self.chars[self.index + i] {
                    return false;
                }
            }
        }
        if token.len() + self.index < self.chars.len() {
            if is_namechar(self.chars[self.index + token.len()]) {
                return false;
            }
        }

        self.push_token(value);

        for _ in 0..token.len()-1 {
            self.nextc();
        }

        return true;
    }


    fn push_token(&mut self, value:TokenValue) {
        self.tokens.push(Token {
            line: self.line,
            col: self.col,
            value: value,
        });
    }

    pub fn tokenize(&mut self) {
        loop {
            if self.state != TokenizerState::Wip {
                return;
            }

            let cur = self.nextc();

            if cur == '\0' {
                self.push_token(TokenValue::Eof);
                self.state = TokenizerState::Done;
                return;
            } else if is_wp(cur) {
                continue;
            } else if cur == '[' {
                self.push_token(TokenValue::LeftSqBrkt);
            } else if cur == ']' {
                self.push_token(TokenValue::RightSqBrkt);
            } else if cur == ':' {
                self.push_token(TokenValue::Colon);
            } else if cur == '.' {
                self.push_token(TokenValue::Dot);
            } else if cur == '|' {
                self.push_token(TokenValue::Pipe);
            } else if cur == '!' {
                self.push_token(TokenValue::Bang);
            } else if cur == ',' {
                self.push_token(TokenValue::Comma);
            } else if cur == '#' {
                let line = self.line;
                let col = self.col;
                let mut comment: Vec<char> = vec![];
                loop {
                    let nextc = self.nextc();

                    if is_eof(nextc) || nextc == '\n' {
                        break;
                    } else {
                        comment.push(nextc);
                    }
                }
                self.tokens.push(Token {
                    line: line,
                    col: col,
                    value: TokenValue::Comment(comment.iter().collect()),
                });

            } else if 
                // here we match for specific keywords
                self.match_and_push_token("-NaN", TokenValue::Number(f64::NAN, None)) ||
                self.match_and_push_token("NaN", TokenValue::Number(f64::NAN, None)) ||
                self.match_and_push_token("-Inf", TokenValue::Number(f64::NEG_INFINITY, None)) ||
                self.match_and_push_token("Inf", TokenValue::Number(f64::INFINITY, None)) ||
                self.match_and_push_token("Pi", TokenValue::Number(std::f64::consts::PI, None)) ||
                self.match_and_push_token("-Pi", TokenValue::Number(-std::f64::consts::PI, None))
            {
                continue;
            } else if is_digit(cur) || (cur == '-' && is_digit(self.peek1())) {
                // here we parse numbers
                let mut num: Vec<char> = vec![];
                let mut unit: Vec<char> = vec![];
                let line = self.line;
                let col = self.col;
                let mut dotcount = 0;
                let mut numcur = cur;
                let mut error = false;
                loop {
                    if numcur != '_' {
                        num.push(numcur);
                    }

                    let nextc = self.peek1();

                    if is_eof(nextc) || is_wp(nextc) || is_num_separator(nextc) {
                        break;
                    } else if is_digit(nextc) || nextc == '_' {
                        numcur = self.nextc();
                    } else if nextc == '.' {
                        dotcount += 1;
                        if dotcount > 1 {
                            self.state = TokenizerState::Error("Too many dots '.' in number".to_owned());
                            error = true;
                            break;
                        }
                        numcur = self.nextc();
                    } else if is_alpha(nextc) {
                        loop {
                            let nextu = self.peek1();
                            if is_alpha(nextu) {
                                unit.push(self.nextc());
                            } else {
                                break;
                            }
                        }
                        break;
                    } else {
                        self.state = TokenizerState::Error("The number contains unexpected characters".to_owned());
                        error = true;
                        break;
                    }
                }
                if !error {
                    let numstr: String = num.iter().collect();
                    let unitstr: String = unit.iter().collect();
                    match numstr.parse::<f64>() {
                        Ok(val) => self.tokens.push(Token {
                            line: line,
                            col: col,
                            value: TokenValue::Number(val, if unitstr.len() > 0 { Some(unitstr) } else {None}),
                        }),
                        Err(e) => self.state = TokenizerState::Error(e.to_string())
                    }
                }
            } else if cur == '-' {
                let mut str: Vec<char> = vec![];
                let line = self.line;
                let col = self.col;
                loop {
                    let nextc = self.peek1();
                    if is_eof(nextc) || is_wp(nextc) || is_dashstr_separator(nextc) {
                        break;
                    }
                    str.push(self.nextc());
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
                let mut error = false;

                loop {
                    let nextc = self.nextc();

                    if is_eof(nextc) {
                        self.state = TokenizerState::Error("End of file in the middle of a string".to_owned());
                        error = true;
                        break;
                    } else if !escape && nextc == delim {
                        break;
                    } else if !escape && nextc == '\\' {
                        escape = true;
                        continue
                    } else if escape {
                        if nextc ==  'n' {
                            str.push('\n');
                        } else if nextc == 't'{
                            str.push('\t');
                        } else {
                            str.push(nextc);
                        }
                        escape = false;
                    } else {
                        str.push(nextc);
                    }
                }
                if !error {
                    self.tokens.push(Token {
                        line: line,
                        col: col,
                        value: TokenValue::String(str.iter().collect()),
                    });
                }
            } else if is_namechar(cur) {
                let mut name: Vec<char> = vec![];
                let line = self.line;
                let col = self.col;
                let mut namecur = cur;
                loop {
                    name.push(namecur);
                    let nextc = self.peek1();
                    if is_eof(nextc) || !is_namechar(nextc) {
                        break;
                    } else {
                        namecur = self.nextc();
                    }
                }
                self.tokens.push(Token {
                    line: line,
                    col: col,
                    value: TokenValue::Name(name.iter().collect()),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_empty() {
        let mut program = Tokenizer::new(String::from(""));
        program.tokenize();
        assert_eq!(program.tokens, vec![Token{line:1, col:1, value: TokenValue::Eof}]);
        assert_eq!(program.state, TokenizerState::Done);
    }
    
    #[test]
    fn test_parse_lb() {
        let mut program = Tokenizer::new(String::from("["));
        program.tokenize();
        assert_eq!(program.tokens, vec![
            Token{line:1, col:1, value: TokenValue::LeftSqBrkt},
            Token{line:1, col:1, value: TokenValue::Eof},
        ]);
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_wp_lb() {
        let mut program = Tokenizer::new(String::from("  ["));
        program.tokenize();
        assert_eq!(program.tokens, vec![
            Token{line:1, col:3, value: TokenValue::LeftSqBrkt},
            Token{line:1, col:3, value: TokenValue::Eof},
        ]);
        assert_eq!(program.state, TokenizerState::Done);
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
                Token{line:1, col:4, value: TokenValue::Eof},
            ]
        );
        assert_eq!(program.state, TokenizerState::Done);
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
                Token{line:2, col:4, value: TokenValue::Eof},
            ]
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_name() {
        let mut program = Tokenizer::new(String::from("name"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Name(String::from("name"))},
                Token{line:1, col:4, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
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
                Token{line:1, col:7, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
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
                Token{line:1, col:16, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
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
                Token{line:1, col:13, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_doctype() {
        let mut program = Tokenizer::new(String::from("-!DOCTYPE"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::String(String::from("!DOCTYPE"))},
                Token{line:1, col:9, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
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
                Token{line:1, col:12, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_wp() {
        let mut program = Tokenizer::new(String::from("'foo \t\nbar' \"foo \t\nbar\""));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::String(String::from("foo \t\nbar"))},
                Token{line:2, col:6, value: TokenValue::String(String::from("foo \t\nbar"))},
                Token{line:3, col:4, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_wp_escaped_sq() {
        let mut program = Tokenizer::new(String::from("'foo \\t\\nbar'"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::String(String::from("foo \t\nbar"))},
                Token{line:1, col:13, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_wp_escaped_dq() {
        let mut program = Tokenizer::new(String::from("\"foo \\t\\nbar\""));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::String(String::from("foo \t\nbar"))},
                Token{line:1, col:13, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_escaped_sq() {
        let mut program = Tokenizer::new(String::from("'foo \\\\ \\' '"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::String(String::from("foo \\ ' "))},
                Token{line:1, col:12, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_escaped_dq() {
        let mut program = Tokenizer::new(String::from("\"foo \\\\ \\\" \""));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::String(String::from("foo \\ \" "))},
                Token{line:1, col:12, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_eof_sq() {
        let mut program = Tokenizer::new(String::from("'foo"));
        program.tokenize();
        assert_eq!(program.tokens, vec![]);
        assert_eq!(program.state, TokenizerState::Error("End of file in the middle of a string".to_owned()));
    }

    #[test]
    fn test_parse_string_eof_dq() {
        let mut program = Tokenizer::new(String::from("\"foo"));
        program.tokenize();
        assert_eq!(program.tokens, vec![]);
        assert_eq!(program.state, TokenizerState::Error("End of file in the middle of a string".to_owned()));
    }

    #[test]
    fn test_parse_num_inf() {
        let mut program = Tokenizer::new(String::from("Inf -Inf"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(f64::INFINITY, None)},
                Token{line:1, col:5, value: TokenValue::Number(f64::NEG_INFINITY, None)},
                Token{line:1, col:8, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_num_42() {
        let mut program = Tokenizer::new(String::from("42"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(42.0, None)},
                Token{line:1, col:2, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_num_pi_digits() {
        let mut program = Tokenizer::new(String::from("3.141592"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(3.141592, None)},
                Token{line:1, col:8, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_num_neg42() {
        let mut program = Tokenizer::new(String::from("-42"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(-42.0, None)},
                Token{line:1, col:3, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_num_neg_pi_digits() {
        let mut program = Tokenizer::new(String::from("-3.141592"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(-3.141592, None)},
                Token{line:1, col:9, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_num_big() {
        let mut program = Tokenizer::new(String::from("9_000_000.000_123"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(9000000.000123, None)},
                Token{line:1, col:17, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_num_dotdot() {
        let mut program = Tokenizer::new(String::from("1.2.3"));
        program.tokenize();
        assert_eq!(program.tokens, vec![]);
        assert_eq!(program.state, TokenizerState::Error("Too many dots '.' in number".to_owned()));
    }

    #[test]
    fn test_parse_num_123xyz() {
        let mut program = Tokenizer::new(String::from("123?,"));
        program.tokenize();
        assert_eq!(program.tokens, vec![]);
        assert_eq!(program.state, TokenizerState::Error("The number contains unexpected characters".to_owned()));
    }

    #[test]
    fn test_parse_num_123unit() {
        let mut program = Tokenizer::new(String::from("123unit"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(123.0, Some("unit".to_owned()))},
                Token{line:1, col:7, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }
    #[test]
    fn test_parse_num_123unit456kg() {
        let mut program = Tokenizer::new(String::from("123unit456kg"));
        // is it weird syntax ? let date sum[2023y4m5d3h25s]
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(123.0, Some("unit".to_owned()))},
                Token{line:1, col:8, value: TokenValue::Number(456.0, Some("kg".to_owned()))},
                Token{line:1, col:12, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }


    #[test]
    fn test_parse_num_mix() {
        let mut program = Tokenizer::new(String::from("1 42 -1 99.234"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Number(1.0, None)},
                Token{line:1, col:3, value: TokenValue::Number(42.0, None)},
                Token{line:1, col:6, value: TokenValue::Number(-1.0, None)},
                Token{line:1, col:9, value: TokenValue::Number(99.234, None)},
                Token{line:1, col:14, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_comment() {
        let mut program = Tokenizer::new(String::from("#!/usr/bin/nope --version=1.0"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Comment("!/usr/bin/nope --version=1.0".to_owned())},
                Token{line:1, col:29, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_comment_complex() {
        let mut program = Tokenizer::new(String::from("foo#comment\nbar"));
        program.tokenize();
        assert_eq!(
            program.tokens,
            vec![
                Token{line:1, col:1, value: TokenValue::Name("foo".to_owned())},
                Token{line:1, col:4, value: TokenValue::Comment("comment".to_owned())},
                Token{line:2, col:1, value: TokenValue::Name("bar".to_owned())},
                Token{line:2, col:3, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_basic_dict() {
        let mut program = Tokenizer::new(String::from("[foo:3.14 bar:'hello']"));
        program.tokenize();
        assert_eq!(
            program.tokens,
             vec![
                Token{line:1, col:1, value: TokenValue::LeftSqBrkt},
                Token{line:1, col:2, value: TokenValue::Name("foo".to_owned())},
                Token{line:1, col:5, value: TokenValue::Colon},
                Token{line:1, col:6, value: TokenValue::Number(3.14, None)},
                Token{line:1, col:11, value: TokenValue::Name("bar".to_owned())},
                Token{line:1, col:14, value: TokenValue::Colon},
                Token{line:1, col:15, value: TokenValue::String("hello".to_owned())},
                Token{line:1, col:22, value: TokenValue::RightSqBrkt},
                Token{line:1, col:22, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_array() {
        let mut program = Tokenizer::new(String::from("['Name' 'Height' 'Weight']"));
        program.tokenize();
        assert_eq!(
            program.tokens,
             vec![
                Token{line:1, col:1, value: TokenValue::LeftSqBrkt},
                Token{line:1, col:2, value: TokenValue::String("Name".to_owned())},
                Token{line:1, col:9, value: TokenValue::String("Height".to_owned())},
                Token{line:1, col:18, value: TokenValue::String("Weight".to_owned())},
                Token{line:1, col:26, value: TokenValue::RightSqBrkt},
                Token{line:1, col:26, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }

    #[test]
    fn test_parse_string_array_regression1() {
        let mut program = Tokenizer::new(String::from(
"[
    headers:
    ['Name' 'Height' 'Weight']
]
"
        ));
        program.tokenize();
        assert_eq!(
            program.tokens,
             vec![
                Token{line:1, col:1, value: TokenValue::LeftSqBrkt},
                Token{line:2, col:5, value: TokenValue::Name("headers".to_owned())},
                Token{line:2, col:12, value: TokenValue::Colon},
                Token{line:3, col:5, value: TokenValue::LeftSqBrkt},
                Token{line:3, col:6, value: TokenValue::String("Name".to_owned())},
                Token{line:3, col:13, value: TokenValue::String("Height".to_owned())},
                Token{line:3, col:22, value: TokenValue::String("Weight".to_owned())},
                Token{line:3, col:30, value: TokenValue::RightSqBrkt},
                Token{line:4, col:1, value: TokenValue::RightSqBrkt},
                Token{line:4, col:2, value: TokenValue::Eof},
            ],
        );
        assert_eq!(program.state, TokenizerState::Done);
    }
}
