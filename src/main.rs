
#[derive(PartialEq, Debug)]
enum TokenValue {
    LeftSqBrkt,
    RightSqBrkt,
    //Colon,
    //Number(f64, String),
    //String(String),
    //Name(String),
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
        fn is_wp(c:char) -> bool {
            return  c == ' ' || c == '\t' || c == '\n'
        }
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
            if self.cur() == '[' {
                self.push_token(TokenValue::LeftSqBrkt);    
            } else if self.cur() == ']' {
                self.push_token(TokenValue::RightSqBrkt);
            }
            self.next();
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
}