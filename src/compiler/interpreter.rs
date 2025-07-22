use crate::compiler::{self, ast::{self, *}};
use std::collections::HashMap;
use std::fmt;
use crate::compiler::value::Value;

// Represents a runtime error that can occur during interpretation
#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error: {}", self.message)
    }
}

// Stores function parameters and body for later execution
#[derive(Clone, Debug)]
pub struct FunctionDef {
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

// The interpreter holds global variables and user-defined functions
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
        // First: collect all functions
        for stmt in stmts {
            if let Stmt::Function(func) = stmt {
                self.functions.insert(func.name.clone(), FunctionDef {
                params: func.params.clone(), // includes name and type
                body: func.body.clone(),
            });

            }
        }

        // Then: look for and run the "main" function
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

    // Executes a block of statements with local environment
    fn run_block(&mut self, stmts: &[Stmt], env: &mut HashMap<String, Value>) -> Result<Option<Value>, RuntimeError> {
        for stmt in stmts {
            match stmt {
                Stmt::Let(name, expr) => {
                    let val = self.eval_expr(expr, env)?;
                    env.insert(name.clone(), val);
                }
                Stmt::Print(expr) => {
                    let val = self.eval_expr(expr, env)?;
                    println!("{}", val.to_string());
                }
                Stmt::Return(Some(expr)) => {
                    let val = self.eval_expr(expr, env)?;
                    return Ok(Some(val));
                }
                Stmt::Return(None) => {
                    return Ok(Some(Value::Number(0))); // default return value
                }
                Stmt::If(condition, then_body, else_body) => {
                    let cond_val = self.eval_expr(condition, env)?;
                    if cond_val.is_truthy() {
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
                    while self.eval_expr(condition, env)?.is_truthy() {
                        if let Some(ret) = self.run_block(body, env)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Stmt::Expression(expr) => {
                    self.eval_expr(expr, env)?;
                }
                Stmt::Function(_) => {} // already handled above
            }
        }
        Ok(None)
    }

    // Evaluates expressions recursively
    fn eval_expr(&mut self, expr: &Expr, env: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
        use BinaryOperator::*;
        use UnaryOperator::*;
        use Value::*;

        match expr {
            Expr::Number(n) => Ok(Number(*n)),
            Expr::Bool(b) => Ok(Bool(*b)),
            Expr::String(s) => Ok(Str(s.clone())),

            Expr::Ident(name) => {
                env.get(name)
                    .cloned()
                    .ok_or_else(|| RuntimeError {
                        message: format!("Undefined variable: {}", name),
                    })
            }

            Expr::BinaryOp(lhs, op, rhs) => {
                let left = self.eval_expr(lhs, env)?;
                let right = self.eval_expr(rhs, env)?;

                match op {
                    Add => match (left, right) {
                        (Number(a), Number(b)) => Ok(Number(a + b)),
                        (Str(a), Str(b)) => Ok(Str(a + &b)),
                        _ => Err(RuntimeError { message: "'+' supports numbers or strings".into() }),
                    },
                    Subtract => match (left, right) {
                        (Number(a), Number(b)) => Ok(Number(a - b)),
                        _ => Err(RuntimeError { message: "'-' only supports numbers".into() }),
                    },
                    Multiply => match (left, right) {
                        (Number(a), Number(b)) => Ok(Number(a * b)),
                        _ => Err(RuntimeError { message: "'*' only supports numbers".into() }),
                    },
                    Divide => match (left, right) {
                        (Number(_), Number(0)) => Err(RuntimeError { message: "Division by zero".into() }),
                        (Number(a), Number(b)) => Ok(Number(a / b)),
                        _ => Err(RuntimeError { message: "'/' only supports numbers".into() }),
                    },
                    Modulo => match (left, right) {
                        (Number(_), Number(0)) => Err(RuntimeError { message: "Modulo by zero".into() }),
                        (Number(a), Number(b)) => Ok(Number(a % b)),
                        _ => Err(RuntimeError { message: "'%' only supports numbers".into() }),
                    },
                    Equal => Ok(Bool(left == right)),
                    NotEqual => Ok(Bool(left != right)),
                    Less => match (left, right) {
                        (Number(a), Number(b)) => Ok(Bool(a < b)),
                        _ => Err(RuntimeError { message: "'<' only supports numbers".into() }),
                    },
                    Greater => match (left, right) {
                        (Number(a), Number(b)) => Ok(Bool(a > b)),
                        _ => Err(RuntimeError { message: "'>' only supports numbers".into() }),
                    },
                    LessEqual => match (left, right) {
                        (Number(a), Number(b)) => Ok(Bool(a <= b)),
                        _ => Err(RuntimeError { message: "'<=' only supports numbers".into() }),
                    },
                    GreaterEqual => match (left, right) {
                        (Number(a), Number(b)) => Ok(Bool(a >= b)),
                        _ => Err(RuntimeError { message: "'>=' only supports numbers".into() }),
                    },
                }
            }

            Expr::UnaryOp(op, expr) => {
                let val = self.eval_expr(expr, env)?;
                match op {
                    Minus => {
                        if let Number(n) = val {
                            Ok(Number(-n))
                        } else {
                            Err(RuntimeError { message: "Unary '-' needs a number".into() })
                        }
                    }
                    Not => Ok(Bool(!val.is_truthy())),
                }
            }

            Expr::Call(name, args) => {
                let func = self.functions.get(name).cloned().ok_or_else(|| RuntimeError {
                    message: format!("Undefined function: {}", name),
                })?;

                if args.len() != func.params.len() {
                    return Err(RuntimeError {
                        message: format!("Function {} expects {} args, got {}", name, func.params.len(), args.len()),
                    });
                }

                let mut local_env = env.clone();
                for (param, arg_expr) in func.params.iter().zip(args.iter()) {
                let val = self.eval_expr(arg_expr, env)?;

                // Type check here:
                match (&param.param_type, &val) {
                    (ParamType::String, Value::Str(_)) => {}
                    (ParamType::Number, Value::Number(_)) => {}
                    (ParamType::Bool,   Value::Bool(_)) => {}
                    (ParamType::Any,    _)              => {}
                    _ => return Err(RuntimeError {
                        message: format!("Type mismatch for parameter '{}'", param.name),
                    }),
                }

                local_env.insert(param.name.clone(), val);
            }


                Ok(self.run_block(&func.body, &mut local_env)?.unwrap_or(Number(0)))
            }
        }
    }
}

// Extension trait to simplify truthiness checks
trait Truthy {
    fn is_truthy(&self) -> bool;
}

impl Truthy for Value {
    fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(true) => true,
            Value::Number(n) => *n != 0,
            Value::Str(s) => !s.is_empty(),
            _ => false,
        }
    }
}