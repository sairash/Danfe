use std::clone;

use thiserror::Error;
use crate::lexer::*;

#[derive(Debug, Error)]
pub enum ExprError {
    #[error("Was expecting {expected:?},  {found:?}")]
    FailedConversion{expected: String, found: String},

    #[error("Only Int or Float is Allowed to be first element")]
    InvalidFirstElement
}

#[derive(Debug)]
pub struct Program {
    pub exprs: Vec<Expr>
}


impl Program{
    pub fn new() -> Program {
        Program{
            exprs: vec![]
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i32),
    FloatingPoint(f32),
    String(String),
    Boolean(bool),
    Symbol(String),
}


#[derive(Debug, Clone)]
pub enum Operator {
    UnaryPass,
    UnaryMinus,
    LogicalNegate,

    Multiply,
    Substract,
    Modulos,
    Division,

    Plus,
    Minus,

    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThantOrEqual,
    Equal,
    NotEqual,

    BooleanAnd,
    BooleanEqual,

    Call(String),
    Index,

    Assignment,
}


#[derive(Debug, Clone)]
pub struct OpExpr {
    pub op: Operator,
    pub args: Vec<Expr>,
}

impl OpExpr {
    pub fn operator(op: Operator, left: Expr, right: Expr) -> OpExpr {
        OpExpr { op, args: vec![left, right]}
    }
    
    pub fn function_op(op: Operator, args: Expr) -> OpExpr {
        OpExpr { op, args: vec![args] }
    }
}


#[derive(Debug, Clone)]
pub enum Expr {
    OpExpr(Box<OpExpr>),
    OpLiteral(Box<Literal>)
}

impl Expr {
    pub fn new_literal(token: TokenType)-> Result<Expr, ExprError>{
        match token {
            TokenType::Numeric { raw, hint }=>{
                match hint {
                    NumericType::Integer => {
                        let number:Result<i32, _> = raw.parse();
                        match number {
                            Ok(number)=>{
                                Ok(Expr::OpLiteral(Box::new(Literal::Integer(number))))
                           },
                           Err(_)=>{
                            Err(ExprError::FailedConversion{
                                expected: "int".to_string(),
                                found: raw
                            })
                           }
                        }
                    },
                    NumericType::FloatingPoint=>{
                        let number:Result<f32, _> = raw.parse();
                        match number {
                           Ok(number)=>{
                               Ok(Expr::OpLiteral(Box::new(Literal::FloatingPoint(number))))
                           },
                           Err(_)=>{
                            Err(ExprError::FailedConversion{
                                expected: "_._".to_string(),
                                found: raw
                            })
                           }
                        }
                    }
                }
            },
            _=>{
                Err(ExprError::InvalidFirstElement)
            }
        }
    }
}