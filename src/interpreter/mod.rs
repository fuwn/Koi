use crate::ast::{Expr, Value, Stmt};
use crate::scanner::Token;
use itertools::{interleave, Itertools};
use std::collections::HashMap;

#[cfg(test)]
mod test;

pub struct Interpreter {
    vars: HashMap<String, Value>,

    capture: bool,
    captured: String,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            vars: HashMap::new(),
            capture: false,
            captured: String::new(),
        }
    }

    /// Executes a series of statements (program)
    pub fn interpret(&mut self, prog: &Vec<Stmt>) {
        for stmt in prog {
            self.exec(stmt);
        }
    }

    /// Executes a single statement
    fn exec(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval(expr);
            },
            Stmt::Print(expr) => {
                let res = self.eval(expr).stringify();

                if self.capture {
                    self.captured.push_str(&res);
                } else {
                    println!("{}", res);
                }
            },
            Stmt::Var { name, initializer } => {
                self.vars.insert(name.to_owned(), match initializer {
                    Some(expr) => self.eval(expr),
                    None => Value::Nil,
                });
            },
            Stmt::If {cond, then_do, else_do} => {
                let cond = self.eval(cond);
                if cond.is_truthy() {
                    self.interpret(then_do);
                } else {
                    self.interpret(else_do);
                }
            },
            _ => unimplemented!(),
        };
    }

    /// Evaluates an expression and returns a value
    fn eval(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Value(value) => value.clone(),
            Expr::Var(name) => self.vars.get(name).and_then(
                |value| Some(value.clone()),
            ).unwrap_or(Value::Nil),
            Expr::Interp { segments, exprs } => {
                let segments = interleave(
                    segments.iter().map(String::clone),
                    exprs.iter()
                        .map(|expr| self.eval(expr))
                        .map(|value| value.stringify()),
                ).collect::<Vec<_>>();

                Value::String(segments.iter().join(""))
            }

            Expr::Unary { op, rhs } => {
                let rhs = self.eval(rhs);

                match op {
                    Token::Plus => rhs,
                    Token::Minus => {
                        if let Value::Num(number) = rhs {
                            Value::Num(-number)
                        } else {
                            panic!("bad operand, expected number");
                        }
                    }
                    _ => panic!("bad op {:?}", op),
                }
            }

            Expr::Binary { lhs, op, rhs } => {
                let lhs = self.eval(lhs);

                match op {
                    Token::PipePipe => {
                        return Value::Bool(lhs.is_truthy() || self.eval(rhs).is_truthy());
                    }
                    Token::AmperAmper => {
                        return Value::Bool(lhs.is_truthy() && self.eval(rhs).is_truthy());
                    }
                    _ => (),
                }

                let rhs = self.eval(rhs);

                match op {
                    Token::Plus |
                    Token::Minus |
                    Token::Star |
                    Token::Slash |
                    Token::Perc |
                    Token::Caret => {
                        if let (Value::Num(lhs), Value::Num(rhs)) = (lhs, rhs) {
                            Value::Num(match op {
                                Token::Plus => lhs + rhs,
                                Token::Minus => lhs - rhs,
                                Token::Star => lhs * rhs,
                                Token::Slash => lhs / rhs,
                                Token::Perc => lhs % rhs,
                                Token::Caret => lhs.powf(rhs),
                                _ => unreachable!(),
                            })
                        } else {
                            panic!("bad operands, expected numbers");
                        }
                    }

                    _ => panic!("bad op {:?}", op),
                }
            }
        }
    }
}
