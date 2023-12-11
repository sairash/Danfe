// extern crate thiserror;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("")]
    FileIo(#[from] io::Error),

    #[error("Was expecting {expected:?}, found {found:?}")]
    MissingExpectedSymbol { expected: TokenType, found: Token },

    #[error("Not a valid number {raw:?}")]
    NumericInvalid { raw: String },

    #[error("Cant't find opening {open:?} symbol for {symbol:?}")]
    MisbalancedBraces { symbol: char, open: char },

    #[error("")]
    UnknownSymbol { symbol: String },

    #[error("There was an unexpected End Of file")]
    UnexpectedEOF,
}

pub type Token = TokenType;

#[derive(Debug, PartialEq, Clone)]
pub enum NumericType {
    Integer,
    FloatingPoint
}


#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    EOF,

    EOL, // End of Line

    /* ([]) */
    Puncutation { raw: char, kind: PunctuationKind },

    String(String),

    /* actions * + */
    Operators(String),

    /* Seq of chars */
    Identifier(String),

    Numeric{raw: String, hint: NumericType},

    Symobl(String),

    Comment
}

#[derive(Debug, PartialEq, Clone)]
pub enum PunctuationKind {
    Open(BalancingDepthType),
    Close(BalancingDepthType),
    Seperator,
}

type BalancingDepthType = i32;

pub struct Lexer<'a> {
    /* human readable */
    pub cur_line: usize,
    pub cur_col: usize,

    /* (raw) code points */
    pub codepoint_offset: usize,

    chars: std::iter::Peekable<std::str::Chars<'a>>,
    balancing_state: std::collections::HashMap<char, BalancingDepthType>,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: &'a str) -> Lexer<'a> {
        Lexer {
            cur_col: 1,
            cur_line: 1,

            codepoint_offset: 0,

            chars: chars.chars().peekable(),
            balancing_state: std::collections::HashMap::new(),
        }
    }

    fn map_balance(c: &char) -> char {
        match c {
            '{' => '}',
            ')' => '(',
            '(' => ')',
            '}' => '{',
            '[' => ']',
            ']' => '[',
            _ => panic!("Balancing Char Not found!")
        }
    }

    fn push_symbol(&mut self, c: &char) -> BalancingDepthType {
        if let Some(v) = self.balancing_state.get_mut(&c) {
            *v += 1;
            *v - 1
        } else {
            self.balancing_state.insert(*c, 1);
            0
        }
    }

    fn pop_symbol(&mut self, c: &char) -> Result<BalancingDepthType, LexerError> {
        if let Some(v) = self.balancing_state.get_mut(&Lexer::map_balance(&c)) {
            if *v >= 1 {
                *v -= 1;
                Ok(*v)
            } else {
                Err(LexerError::MisbalancedBraces {
                    symbol: *c,
                    open: Lexer::map_balance(&c)
                })
            }
        } else {
            Err(LexerError::MisbalancedBraces {
                symbol: *c,
                open: Lexer::map_balance(&c)
            })
        }
    }

    fn match_number(&mut self, start: char) ->  Result<TokenType, LexerError> {
        let mut seen_dot = false;
        let radix = 10;

        let mut num = start.to_string();

        if start == '.' {
            seen_dot = true;
        }


        loop{
            match self.chars.peek() {
                Some(c) if *c == '.' && !seen_dot => {
                    num.push(*c);
                    self.consume_char();
                    seen_dot = true
                }
                Some(c) if c.is_digit(radix) => {
                    num.push(*c);
                    self.consume_char();
                }
                Some(c) if *c == '_' => {
                    self.consume_char();
                }
                Some(c) if c.is_ascii_alphabetic() || c.is_digit(10) => {
                    num.push(*c);
                    self.consume_char();
                    return Err(LexerError::NumericInvalid { raw: num.to_string() });
                }
                _ => {
                    break Ok(TokenType::Numeric {raw: num, hint: if seen_dot {NumericType::FloatingPoint} else {NumericType::Integer}});
                }
            }
        }

    }

    fn match_string(&mut self, start: char)-> Result<TokenType, LexerError> {
        let mut return_string = String::new();

        loop {
            match self.chars.next() {
                Some(c) if c == start  => {
                    break Ok(TokenType::String(return_string));
                }
                Some(c) if c == '\\'  => {
                    match self.chars.peek(){
                        Some(s) if *s == '"' =>{
                            return_string.push(*s);
                            self.consume_char();
                        }
                        _=>{}
                    }

                }
            
                Some(c) => {
                    return_string.push(c);
                }

                None => {
                    break Err(LexerError::UnknownSymbol { symbol: "Unexpected EOF".to_string() })
                }
            }
        }

    }

    fn match_identifier(&mut self, start: char) -> Result<TokenType, LexerError> {
        let mut return_string = start.to_string();

        loop {
            match self.chars.peek() {
                Some(c) if c.is_ascii_alphabetic() || c.is_digit(10) ||*c == '_'  => {
                    return_string.push(*c);
                    self.consume_char();
                }
                _ => {
                    break Ok(self.match_symbol(return_string));
                }
            }
        }

    }

    fn match_symbol(&mut self, identifier: String) -> TokenType {
        if match identifier.as_ref() {
            "false" | "true" | "proc" | "if" | "else" | "loop" | "break" | "print" | "input" => true,
            _ => false
        }{
            TokenType::Symobl(identifier)
        }else {
            TokenType::Identifier(identifier)
        }
    }

    fn match_operator(&mut self, start: char) -> Result<TokenType, LexerError> {
        let mut return_operators = start.to_string();

        match self.chars.peek() {
            Some(c) if *c == '=' && (start == '+' || start == '-' || start == '=' || start == '>' || start == '<' || start == '%')   => {
                return_operators.push(*c);
                self.consume_char();
            }
            Some(c) if *c == '|' && (start == '|')   => {
                return_operators.push(*c);
                self.consume_char();
            }
            Some(c) if *c == '&' && (start == '&')   => {
                return_operators.push(*c);
                self.consume_char();
            }
            Some(c) if *c == '+' && start == '+' => {
                return_operators.push(*c);
                self.consume_char();
            }
            Some(c) if *c == '-' && start == '-' => {
                return_operators.push(*c);
                self.consume_char();
            }
            None => return Err(LexerError::UnexpectedEOF),
            _ => {
                return Ok(TokenType::Operators(return_operators));
            }

        }
        return Ok(TokenType::Operators(return_operators));

    }


    fn transform_to_type(&mut self, c: char) -> Result<TokenType, LexerError> {
        match c {
            '(' | '[' | '{' => Ok(TokenType::Puncutation {
                raw: c,
                kind: PunctuationKind::Open(self.push_symbol(&c)),
            }),
            ')' | ']' | '}' => Ok(TokenType::Puncutation {
                raw: c,
                kind: PunctuationKind::Close(self.pop_symbol(&c)?),
            }),
            '0' ..= '9' | '.'=> self.match_number(c),
            '"' | '\'' => self.match_string(c),
            '+' | '-' | '*' | '/' | '\\' | '%' |'=' | '|' | '&' | '<' | '>' => self.match_operator(c),
            ','| ';' => Ok(TokenType::Puncutation {
                raw: c,
                kind: PunctuationKind::Seperator
            }),
            '#' => Ok({
                loop {
                    match self.chars.next() {
                        Some(c) => {
                            if c == '\n'  {
                                break;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
                TokenType::Comment

            }),
            '\n' => Ok(TokenType::EOL),
            'a' ..= 'z' | 'A' ..= 'Z'=> self.match_identifier(c), 
            _ => Err(LexerError::UnknownSymbol {
                symbol: c.to_string(),
            }),
        }
    }

    pub fn consume_char(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(c) => {
                self.cur_col += 1;

                if c == '\n' {
                    self.cur_line += 1;
                    self.cur_col += 1;
                }

                self.codepoint_offset += 1;
                Some(c)
            }
            None => None,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if !c.is_whitespace() || *c == '\n' {
                break;
            }
            self.consume_char();
        }
    }

    pub fn next_token(&mut self) -> Result<TokenType, LexerError> {
        self.skip_whitespace();
        
        if let Some(c) = self.consume_char() {
            self.transform_to_type(c)
        } else {
            Ok(TokenType::EOF)
        }
    }

    pub fn peek_next_token(&mut self) -> Result<TokenType, LexerError>{

        let cur_col = self.cur_col.clone();
        let cur_line = self.cur_line.clone();
        let codepoint_offset = self.codepoint_offset.clone();

        if let Some(c) = self.consume_char() { 
            let t_to_type = self.transform_to_type(c);
            self.cur_col = cur_col;
            self.cur_line = cur_line;
            self.codepoint_offset = codepoint_offset;

            t_to_type
        } else {
            Ok(TokenType::EOF)
        }
    }
}
