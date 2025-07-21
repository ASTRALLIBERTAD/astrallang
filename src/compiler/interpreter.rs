use crate::compiler::{self, ast::*};
use std::collections::HashMap;
use std::fmt;
use crate::compiler::value::Value;


#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error: {}", self.message)
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDef {
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

pub struct Interpreter {
    globals: HashMap<String, Value>,
    functions: HashMap<String, FunctionDef>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> Result<(), RuntimeError> {
        // First pass: collect function definitions
        for stmt in stmts {
            if let Stmt::Function(func) = stmt {
                self.functions.insert(func.name.clone(), FunctionDef {
                    params: func.params.clone(),
                    body: func.body.clone(),
                });
            }
        }

        // Second pass: look for and execute main function
        if let Some(main_func) = self.functions.get("main").cloned() {
            if !main_func.params.is_empty() {
                return Err(RuntimeError {
                    message: "Main function cannot have parameters".to_string(),
                });
            }
            let mut env = HashMap::new();
            self.run_block(&main_func.body, &mut env)?;
        } else {
            return Err(RuntimeError {
                message: "No main function found".to_string(),
            });
        }

        Ok(())
    }

    fn run_block(&mut self, stmts: &[Stmt], env: &mut HashMap<String, Value>) -> Result<Option<Value>, RuntimeError> {
        for stmt in stmts {
            match stmt {
                Stmt::Let(name, expr) => {
                    let val = self.eval_expr(expr, env)?;
                    env.insert(name.clone(), val);
                }
                Stmt::Print(expr) => {
                    let val = self.eval_expr(expr, env)?;
                    match val {
                        Value::Number(n) => println!("{}", n),
                        Value::Bool(b) => println!("{}", b),
                        Value::Str(s) => println!("{}", s),
                        Value::Null => println!("null"),
                    }
                }
                Stmt::Return(Some(expr)) => {
                    let val = self.eval_expr(expr, env)?;
                    return Ok(Some(val));
                }
                Stmt::Return(None) => {
                    return Ok(Some(Value::Number(0)));
                }
                Stmt::If(condition, then_body, else_body) => {
                    let cond_val = self.eval_expr(condition, env)?;
                    if match cond_val {
                        Value::Bool(true) => true,
                        Value::Number(n) => n != 0,
                        _ => false,
                    } {
                        if let Some(ret) = self.run_block(then_body, env)? {
                            return Ok(Some(ret));
                        }
                    } else if let Some(else_body) = else_body {
                        if let Some(ret) = self.run_block(else_body, env)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Stmt::While(condition, body) => {
                    while {
                        let cond_val = self.eval_expr(condition, env)?;
                        match cond_val {
                            Value::Bool(b) => b,
                            Value::Number(n) => n != 0,
                            _ => false,
                        }
                    } {
                        if let Some(ret) = self.run_block(body, env)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Stmt::Expression(expr) => {
                    self.eval_expr(expr, env)?;
                }
                Stmt::Function(_) => {
                    // Function definitions are handled in the first pass
                }
            }
        }
        Ok(None)
    }

    fn eval_expr(&mut self, expr: &Expr, env: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Number(n) => Ok(compiler::value::Value::Number(*n)),
            Expr::Ident(name) => {
                env.get(name)
                    .cloned()
                    .ok_or_else(|| RuntimeError {
                        message: format!("Undefined variable: {}", name),
                    })
            }
            Expr::String(s) => Ok(Value::Str(s.clone())),
            Expr::BinaryOp(lhs, op, rhs) => {
                let left = self.eval_expr(lhs, env)?;
                let right = self.eval_expr(rhs, env)?;
                use BinaryOperator::*;
                use Value::*;

                match op {
                    Add => match (left, right) {
                        (Number(a), Number(b)) => Ok(Number(a + b)),
                        (Str(a), Str(b)) => Ok(Str(a + &b)),
                        _ => Err(RuntimeError { message: "Type error: unsupported operands for '+'".into() }),
                    },
                    Subtract => match (left, right) {
                        (Number(a), Number(b)) => Ok(Number(a - b)),
                        _ => Err(RuntimeError { message: "Type error: '-' only supports numbers".into() }),
                    },
                    Multiply => match (left, right) {
                        (Number(a), Number(b)) => Ok(Number(a * b)),
                        _ => Err(RuntimeError { message: "Type error: '*' only supports numbers".into() }),
                    },
                    Divide => match (left, right) {
                        (Number(_), Number(0)) => Err(RuntimeError { message: "Division by zero".into() }),
                        (Number(a), Number(b)) => Ok(Number(a / b)),
                        _ => Err(RuntimeError { message: "Type error: '/' only supports numbers".into() }),
                    },
                    Modulo => match (left, right) {
                        (Number(_), Number(0)) => Err(RuntimeError { message: "Modulo by zero".into() }),
                        (Number(a), Number(b)) => Ok(Number(a % b)),
                        _ => Err(RuntimeError { message: "Type error: '%' only supports numbers".into() }),
                    },
                    Equal => Ok(Bool(left == right)),
                    NotEqual => Ok(Bool(left != right)),
                    Less => match (left, right) {
                        (Number(a), Number(b)) => Ok(Bool(a < b)),
                        _ => Err(RuntimeError { message: "Type error: '<' only supports numbers".into() }),
                    },
                    Greater => match (left, right) {
                        (Number(a), Number(b)) => Ok(Bool(a > b)),
                        _ => Err(RuntimeError { message: "Type error: '>' only supports numbers".into() }),
                    },
                    LessEqual => match (left, right) {
                        (Number(a), Number(b)) => Ok(Bool(a <= b)),
                        _ => Err(RuntimeError { message: "Type error: '<=' only supports numbers".into() }),
                    },
                    GreaterEqual => match (left, right) {
                        (Number(a), Number(b)) => Ok(Bool(a >= b)),
                        _ => Err(RuntimeError { message: "Type error: '>=' only supports numbers".into() }),
                    },
                }
            }

            Expr::UnaryOp(op, expr) => {
                let val = self.eval_expr(expr, env)?;
                match op {
                    UnaryOperator::Minus => {
                        if let Value::Number(n) = val {
                            Ok(Value::Number(-n))
                        } else {
                            Err(RuntimeError { message: "Unary '-' applied to a non-number value".to_string() })
                        }
                    },
                    UnaryOperator::Not => Ok(if val == compiler::value::Value::Number(0) { compiler::value::Value::Number(1) } else { compiler::value::Value::Number(0) }),
                }
            }
            Expr::Call(name, args) => {
                let func = self.functions.get(name).cloned().ok_or_else(|| RuntimeError {
                    message: format!("Undefined function: {}", name),
                })?;
                
                if args.len() != func.params.len() {
                    return Err(RuntimeError {
                        message: format!(
                            "Function {} expects {} arguments, got {}",
                            name,
                            func.params.len(),
                            args.len()
                        ),
                    });
                }
                
                let mut local_env = env.clone();
                for (param, arg) in func.params.iter().zip(args.iter()) {
                    let val = self.eval_expr(arg, env)?;
                    local_env.insert(param.clone(), val);
                }
                
                match self.run_block(&func.body, &mut local_env)? {
                    Some(val) => Ok(val),
                    None => Ok(compiler::value::Value::Number(0)), // Functions without explicit return return 0
                }
            }

            Expr::Bool(b) => {Ok(if *b { compiler::value::Value::Bool(true) } else { compiler::value::Value::Bool(false) })}
            Expr::String(s) => {
                // For simplicity, we just return the length of the string as its value
                // In a real interpreter, you might want to handle strings differently
                Ok(compiler::value::Value::Str(s.to_string()))
            }
        }
    }
}
