use crate::ast::*;
use crate::lexer::*;

use thiserror::Error;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: TokenType,
}

#[derive(Error, Debug)]
pub enum ParserError {
    
}


impl <'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Parser {
        let  start_val = lexer.next_token(); 
        match start_val {
           Ok(current_token)=>{
               Parser { lexer, current_token} 
           },
           Err(e) => {
            panic!("{}", e)
           }
        }
    }
    
    fn eat(&mut self, expected: TokenType) {
        if self.current_token == expected {
            let  start_val = self.lexer.next_token(); 
            match start_val {
                Ok(current_token)=>{
                    self.current_token = current_token;
                },
                Err(e) => {
                    panic!("Error -> {}", e)
                }
            }
        } else {
            panic!("Expected {:?}, found {:?}", expected, self.current_token);
        }
    }


    fn next(&mut self) -> Expr {
        match self.current_token.clone() {
            TokenType::Numeric{ raw, hint } => {
                let literal_with_error = Expr::new_literal(self.current_token.clone());
                self.eat(TokenType::Numeric{raw, hint});
                match literal_with_error {
                    Ok(lit)=>{
                        return lit
                    },
                    Err(e)=>{
                        panic!("{}", e)
                    }
                }
            }
            _ => panic!("Unexpected token in factor: {:?}", self.current_token),
        }
    }

    fn parse_expression(&mut self) -> Expr {
        let mut left = self.parse_term();

        while let token = self.current_token.clone() {
            match token {
                Token::Operators(op) => {
                    if op == "+".to_string() {
                        self.eat(TokenType::Operators(op));
                    let right = self.parse_term();
                        left = Expr::OpExpr(Box::new(OpExpr::operator(Operator::Plus, left, right)))
                    }else if op == "-".to_string() {
                        self.eat(TokenType::Operators(op));
                        let right = self.parse_term();
                        left = Expr::OpExpr(Box::new(OpExpr::operator(Operator::Substract, left, right))) 
                    }else {
                        break;
                    }
                }
                _ => break,
            }
        }

        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        // println!("{:?}", left);

        while let token = self.current_token.clone() {
            match token {
                Token::Operators(op) => {
                    if op == "/".to_string() {
                        self.eat(TokenType::Operators(op));
                        let right = self.parse_factor();
                        left = Expr::OpExpr(Box::new(OpExpr::operator(Operator::Division, left, right))) 
                    }else if op == "*".to_string() {
                        self.eat(TokenType::Operators(op));
                        let right = self.parse_factor();
                        left = Expr::OpExpr(Box::new(OpExpr::operator(Operator::Multiply, left, right)))
                    }else if op == "%".to_string() {
                        self.eat(TokenType::Operators(op));
                        let right = self.parse_factor();
                        left = Expr::OpExpr(Box::new(OpExpr::operator(Operator::Modulos, left, right))) 
                    }else if op == "==".to_string() {
                        self.eat(TokenType::Operators(op));
                        let right = self.parse_factor();
                        left = Expr::OpExpr(Box::new(OpExpr::operator(Operator::Equal, left, right))) 
                    }else {
                        break;
                    }
                }
                _ => {break},
            }
        }

        left
    }


    fn parse_factor(&mut self) -> Expr {
        match self.current_token.clone() {
            TokenType::Numeric{raw, hint} => {
               let literal_with_error = Expr::new_literal(self.current_token.clone());
                match literal_with_error {
                    Ok(lit)=>{
                        self.eat(TokenType::Numeric { raw, hint});
                        lit
                    },
                    Err(e)=>{
                        panic!("Panic Factor: {}", e)
                    }
                }
            },
            Token::Puncutation{raw, kind} => {
                if raw == '('{ 
                    match kind {
                        PunctuationKind::Open(depth) => {        
                            self.eat(TokenType::Puncutation { raw: raw, kind: kind });
                            let expr = self.parse_expression();
                            self.remove_eol();
                            assert_eq!(self.current_token, Token::Puncutation{raw: ')', kind:PunctuationKind::Close(depth)});
                            self.eat(TokenType::Puncutation { raw: ')', kind: PunctuationKind::Close(depth) });
                            expr
                        }
                        _=>{
                            panic!("Not Allowed token: {:?}", self.current_token)
                        }
                    }
                }else{
                    panic!("Error on token: {:?}", self.current_token)
                }
            },
            TokenType::Identifier(i) => {
                if i.is_ascii(){
                    self.eat(TokenType::Identifier(i.clone()));
                    let op_symbol = Expr::OpLiteral(Box::new(Literal::Symbol(i.clone())));
                    
                    // println!("{:?}", op_symbol);
                    match self.current_token.clone() {
                        TokenType::Operators(op) => {
                            if op == String::from("=") {
                                self.eat(TokenType::Operators("=".to_string()));
                                let next_expers = self.parse_expression();
                                Expr::OpExpr(Box::new(OpExpr { op: Operator::Assignment, args: vec![op_symbol, next_expers] }))
                            }else {
                                op_symbol
                            }
                        },
                        TokenType::EOF | TokenType::EOL => {
                            op_symbol
                        },
                        _ => {
                            panic!("The Token type {:?} used after identifier {:?} is not correct.", self.current_token, i)
                        }
                    }
                }else {
                    panic!("The {:?} identefier is not allowed.", i)
                }
            },
            TokenType::Symobl(sym) => {
                self.eat(TokenType::Symobl(sym.clone()));

                match sym.as_ref() {
                    "print" => {
                        self.parse_function("print")
                    }
                    "if" => {
                        self.parse_function("if")
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            },
            TokenType::Comment =>{
                self.eat(TokenType::Comment);
                Expr::OPComment
            }
            TokenType::EOL => {
                self.eat(TokenType::EOL);
                self.parse_expression()
            }
            _ => panic!("Unexpected token: {:?}", self.current_token),
        }
    }

    fn remove_eol(&mut self){
        loop {
            match self.current_token {
                TokenType::EOL=>{
                    self.eat(TokenType::EOL)
                }
                (_)=>{
                    break;
                }
            }
        }
    }

    fn parse_function(&mut self, function_name: &str) -> Expr {
        match self.current_token.clone() {
            TokenType::Puncutation { raw, kind } => {
                if raw == '(' {
                    match kind {
                        PunctuationKind::Open(depth) => {
                            self.eat(TokenType::Puncutation { raw: raw, kind: kind });
                            let expr = self.parse_expression();
                            self.remove_eol();
                            assert_eq!(self.current_token, Token::Puncutation{raw: ')', kind:PunctuationKind::Close(depth)});
                            self.eat(TokenType::Puncutation { raw: ')', kind: PunctuationKind::Close(depth) });
                            match self.current_token.clone()  {
                                TokenType::EOL =>{
                                    self.eat(TokenType::EOL);
                                    Expr::OpExpr(Box::new(OpExpr::function_op(Operator::Call(function_name.to_string()), expr)))
                                }
                                TokenType::EOF =>{
                                    self.eat(TokenType::EOF);
                                    Expr::OpExpr(Box::new(OpExpr::function_op(Operator::Call(function_name.to_string()), expr)))
                                }
                                TokenType::Puncutation { raw, kind }=>{
                                    if raw == '{' {
                                        match kind {
                                            PunctuationKind::Open(depth) =>{
                                                self.eat(TokenType::Puncutation { raw: raw, kind: kind });
                                                let new_expr = self.parse_expression();
                                                self.remove_eol();
                                                assert_eq!(self.current_token, Token::Puncutation{raw: '}', kind:PunctuationKind::Close(depth)});
                                                self.eat(TokenType::Puncutation { raw: '}', kind: PunctuationKind::Close(depth) });
                                                Expr::OpExpr(Box::new(OpExpr::function_op(Operator::Define(expr), new_expr)))
                                                }
                                            _=> unimplemented!()
                                        }
                                    }else if  raw == '}' {
                                        expr
                                    }else{
                                        panic!("This is not correct {}", raw)
                                    }
                                }
                                _ =>unimplemented!()
                            }
                        }
                        PunctuationKind::Seperator =>{
                            self.eat(TokenType::Puncutation { raw: raw, kind: kind });
                            self.parse_expression()
                        }
                        _=>{panic!("Print is a function, use it as print(\"Hello World\")")}
                    }
                }else {
                    unimplemented!()
                }
            }
            _ => {panic!("Print is a function, use it as print(\"Hello World\")")}
        }
    }

    pub fn walk(&mut self) -> Program { 
        let mut program = Program::new();
        
        let mut expressions = self.parse_expression();
        loop {
            match self.current_token {
                TokenType::EOL => {
                    self.eat(TokenType::EOL);
                }
                TokenType::EOF => {
                    program.exprs.push(expressions);
                    break;
                },
                _ => {
                    program.exprs.push(expressions);
                    if self.current_token == TokenType::EOF {
                        break;
                    }
                    expressions = self.parse_expression();
                }
            }
        }
        program
    }
}

