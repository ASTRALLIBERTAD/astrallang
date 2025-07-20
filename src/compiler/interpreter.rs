use crate::compiler::ast::*;
use std::collections::HashMap;
use std::fmt;

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
    globals: HashMap<String, i64>,
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

    fn run_block(&mut self, stmts: &[Stmt], env: &mut HashMap<String, i64>) -> Result<Option<i64>, RuntimeError> {
        for stmt in stmts {
            match stmt {
                Stmt::Let(name, expr) => {
                    let val = self.eval_expr(expr, env)?;
                    env.insert(name.clone(), val);
                }
                Stmt::Print(expr) => {
                    let val = self.eval_expr(expr, env)?;
                    println!("{}", val);
                }
                Stmt::Return(Some(expr)) => {
                    let val = self.eval_expr(expr, env)?;
                    return Ok(Some(val));
                }
                Stmt::Return(None) => {
                    return Ok(Some(0));
                }
                Stmt::If(condition, then_body, else_body) => {
                    let cond_val = self.eval_expr(condition, env)?;
                    if cond_val != 0 {
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
                    while self.eval_expr(condition, env)? != 0 {
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

    fn eval_expr(&mut self, expr: &Expr, env: &HashMap<String, i64>) -> Result<i64, RuntimeError> {
        match expr {
            Expr::Number(n) => Ok(*n),
            Expr::Ident(name) => {
                env.get(name)
                    .or_else(|| self.globals.get(name))
                    .copied()
                    .ok_or_else(|| RuntimeError {
                        message: format!("Undefined variable: {}", name),
                    })
            }
            Expr::BinaryOp(lhs, op, rhs) => {
                let left = self.eval_expr(lhs, env)?;
                let right = self.eval_expr(rhs, env)?;
                match op {
                    BinaryOperator::Add => Ok(left + right),
                    BinaryOperator::Subtract => Ok(left - right),
                    BinaryOperator::Multiply => Ok(left * right),
                    BinaryOperator::Divide => {
                        if right == 0 {
                            Err(RuntimeError {
                                message: "Division by zero".to_string(),
                            })
                        } else {
                            Ok(left / right)
                        }
                    }
                    BinaryOperator::Modulo => {
                        if right == 0 {
                            Err(RuntimeError {
                                message: "Modulo by zero".to_string(),
                            })
                        } else {
                            Ok(left % right)
                        }
                    }
                    BinaryOperator::Equal => Ok(if left == right { 1 } else { 0 }),
                    BinaryOperator::NotEqual => Ok(if left != right { 1 } else { 0 }),
                    BinaryOperator::Less => Ok(if left < right { 1 } else { 0 }),
                    BinaryOperator::Greater => Ok(if left > right { 1 } else { 0 }),
                    BinaryOperator::LessEqual => Ok(if left <= right { 1 } else { 0 }),
                    BinaryOperator::GreaterEqual => Ok(if left >= right { 1 } else { 0 }),
                }
            }
            Expr::UnaryOp(op, expr) => {
                let val = self.eval_expr(expr, env)?;
                match op {
                    UnaryOperator::Minus => Ok(-val),
                    UnaryOperator::Not => Ok(if val == 0 { 1 } else { 0 }),
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
                    None => Ok(0), // Functions without explicit return return 0
                }
            }
        }
    }
}
