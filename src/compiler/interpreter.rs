// src/compiler/interpreter.rs
// Refactored version using macros

use crate::compiler::ast::*;
use crate::compiler::builtins::BuiltinRegistry;
use std::collections::HashMap;
use std::fmt;

// Import macros
#[macro_use]
use crate::compiler::macros::*;

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Clone, Debug)]
pub struct FunctionDef {
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

pub struct Interpreter {
    globals: HashMap<String, Value>,
    functions: HashMap<String, FunctionDef>,
    builtins: BuiltinRegistry,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: HashMap::new(),
            functions: HashMap::new(),
            builtins: BuiltinRegistry::new(),
        }
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> Result<()> {
        // First pass: collect all function definitions
        for stmt in stmts {
            if let Stmt::Function(func) = stmt {
                self.functions.insert(
                    func.name.clone(),
                    FunctionDef {
                        params: func.params.clone(),
                        body: func.body.clone(),
                    },
                );
            }
        }

        // Look for and execute main function
        if let Some(main) = self.functions.get("main").cloned() {
            let mut env = HashMap::new();
            self.call_function(&main, vec![], &mut env)?;
            Ok(())
        } else {
            Err(RuntimeError {
                message: "No main function found".into(),
            })
        }
    }

    fn call_function(
        &mut self,
        func: &FunctionDef,
        args: Vec<Value>,
        parent_env: &mut HashMap<String, Value>,
    ) -> Result<Value> {
        if func.params.len() != args.len() {
            return Err(RuntimeError {
                message: format!(
                    "Function expected {} arguments, got {}",
                    func.params.len(),
                    args.len()
                ),
            });
        }

        // Create new scope with function parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            let converted_arg = self.convert_value_to_param_type(arg, &param.param_type)?;
            parent_env.insert(param.name.clone(), converted_arg);
        }

        // Execute function body
        match self.run_block(&func.body, parent_env)? {
            Some(val) => Ok(val),
            None => Ok(Value::I32(0)),
        }
    }

    fn convert_value_to_param_type(&self, value: &Value, param_type: &ParamType) -> Result<Value> {
        match param_type {
            ParamType::Number(target_type) => {
                if let Some(converted) = value.cast_to(target_type) {
                    Ok(converted)
                } else {
                    self.cast_value(value, target_type)
                }
            }
            ParamType::String => match value {
                Value::String(_) => Ok(value.clone()),
                _ => Ok(Value::String(value_to_string!(value))),
            },
            ParamType::Bool => match value {
                Value::Bool(_) => Ok(value.clone()),
                _ => Ok(Value::Bool(is_truthy!(value))),
            },
            ParamType::Any => Ok(value.clone()),
        }
    }

    fn run_block(
        &mut self,
        stmts: &[Stmt],
        env: &mut HashMap<String, Value>,
    ) -> Result<Option<Value>> {
        for stmt in stmts {
            match self.eval_stmt(stmt, env)? {
                Some(val) => return Ok(Some(val)),
                None => continue,
            }
        }
        Ok(None)
    }

    fn eval_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
    ) -> Result<Option<Value>> {
        match stmt {
            Stmt::Let(name, type_hint, expr) => {
                let val = self.eval_expr(expr, env)?;
                
                let final_val = if *type_hint != Type::Any {
                    self.cast_value(&val, type_hint)?
                } else {
                    val
                };
                
                env.insert(name.clone(), final_val);
                Ok(None)
            }
            Stmt::Print(expr) => {
                let val = self.eval_expr(expr, env)?;
                println!("{}", value_to_string!(&val));
                Ok(None)
            }
            Stmt::Return(Some(expr)) => {
                let val = self.eval_expr(expr, env)?;
                Ok(Some(val))
            }
            Stmt::Return(None) => Ok(Some(Value::I32(0))),
            Stmt::If(cond, then_body, else_body) => {
                if is_truthy!(&self.eval_expr(cond, env)?) {
                    self.run_block(then_body, env)
                } else if let Some(else_body) = else_body {
                    self.run_block(else_body, env)
                } else {
                    Ok(None)
                }
            }
            Stmt::While(cond, body) => {
                while is_truthy!(&self.eval_expr(cond, env)?) {
                    if let Some(val) = self.run_block(body, env)? {
                        return Ok(Some(val));
                    }
                }
                Ok(None)
            }
            Stmt::Block(stmts) => {
                let mut block_env = env.clone();
                self.run_block(stmts, &mut block_env)
            }
            Stmt::Expression(expr) => {
                self.eval_expr(expr, env)?;
                Ok(None)
            }
            Stmt::Function(_) => Ok(None),
        }
    }

    fn pattern_matches(
        &mut self,
        pat: &Pattern,
        val: &Value,
        env: &mut HashMap<String, Value>,
    ) -> Result<bool> {
        match pat {
            Pattern::Value(pv) => Ok(pv == val),
            Pattern::Wildcard => Ok(true),
            Pattern::Identifier(name) => {
                env.insert(name.clone(), val.clone());
                Ok(true)
            }
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &mut HashMap<String, Value>,
    ) -> Result<Value> {
        match expr {
            Expr::Match(expr, arms) => {
                let value = self.eval_expr(expr, env)?;
                for (pat, result_expr) in arms {
                    if self.pattern_matches(pat, &value, env)? {
                        return self.eval_expr(result_expr, env);
                    }
                }
                Err(RuntimeError {
                    message: "No match arm matched".to_string(),
                })
            }
            Expr::Literal(val) => Ok(val.clone()),
            Expr::Ident(name) => {
                env.get(name)
                    .or_else(|| self.globals.get(name))
                    .cloned()
                    .ok_or_else(|| RuntimeError {
                        message: format!("Undefined variable: {}", name),
                    })
            }
            Expr::BinaryOp(lhs, op, rhs) => {
                let left = self.eval_expr(lhs, env)?;
                let right = self.eval_expr(rhs, env)?;
                self.eval_binary_op(&left, op, &right)
            }
            Expr::UnaryOp(op, expr) => {
                let val = self.eval_expr(expr, env)?;
                self.eval_unary_op(op, &val)
            }
            Expr::Cast(expr, target_type) => {
                let val = self.eval_expr(expr, env)?;
                self.cast_value(&val, target_type)
            }
            Expr::Call(name, args) => {
                let mut evaluated_args = Vec::<Value>::new();
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, env)?);
                }

                // Check builtin functions first
                if let Some(result) = self.builtins.call(name, evaluated_args.clone()) {
                    return result;
                }

                // Look up user-defined function
                let func = self
                    .functions
                    .get(name)
                    .ok_or_else(|| RuntimeError {
                        message: format!("Function '{}' not found", name),
                    })?
                    .clone();

                let mut temp_env = env.clone();
                self.call_function(&func, evaluated_args, &mut temp_env)
            }
        }
    }

    fn eval_binary_op(
        &self,
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> Result<Value> {
        // Try to promote types to a common type
        let (promoted_left, promoted_right) = self.promote_numeric_types(left, right)?;
        
        use BinaryOperator::*;
        
        match op {
            Add => {
                // Special case: string concatenation
                if let (Value::String(a), Value::String(b)) = (&promoted_left, &promoted_right) {
                    return Ok(Value::String(format!("{}{}", a, b)));
                }
                impl_arithmetic_op!(&promoted_left, &promoted_right, +)
            }
            Subtract => impl_arithmetic_op!(&promoted_left, &promoted_right, -),
            Multiply => impl_arithmetic_op!(&promoted_left, &promoted_right, *),
            Divide => {
                // Check for division by zero
                if self.is_zero(&promoted_right) {
                    return Err(RuntimeError {
                        message: "Division by zero".into(),
                    });
                }
                impl_arithmetic_op!(&promoted_left, &promoted_right, /)
            }
            Modulo => {
                if self.is_zero(&promoted_right) {
                    return Err(RuntimeError {
                        message: "Modulo by zero".into(),
                    });
                }
                impl_arithmetic_op!(&promoted_left, &promoted_right, %)
            }
            Equal => Ok(Value::Bool(promoted_left == promoted_right)),
            NotEqual => Ok(Value::Bool(promoted_left != promoted_right)),
            Less => impl_comparison_op!(&promoted_left, &promoted_right, <),
            Greater => impl_comparison_op!(&promoted_left, &promoted_right, >),
            LessEqual => impl_comparison_op!(&promoted_left, &promoted_right, <=),
            GreaterEqual => impl_comparison_op!(&promoted_left, &promoted_right, >=),
        }
    }

    fn is_zero(&self, val: &Value) -> bool {
        match val {
            Value::I8(0) | Value::I16(0) | Value::I32(0) | Value::I64(0) | Value::I128(0) => true,
            Value::U8(0) | Value::U16(0) | Value::U32(0) | Value::U64(0) | Value::U128(0) => true,
            Value::F32(n) => *n == 0.0,
            Value::F64(n) => *n == 0.0,
            Value::Usize(0) => true,
            _ => false,
        }
    }

    fn promote_numeric_types(&self, left: &Value, right: &Value) -> Result<(Value, Value)> {
        // If same types, no promotion needed
        if std::mem::discriminant(left) == std::mem::discriminant(right) {
            return Ok((left.clone(), right.clone()));
        }

        // Promotion rules (simplified)
        match (left, right) {
            // Integer promotions to larger types
            (Value::I8(a), Value::I16(b)) => Ok((Value::I16(*a as i16), Value::I16(*b))),
            (Value::I16(a), Value::I8(b)) => Ok((Value::I16(*a), Value::I16(*b as i16))),
            (Value::I8(a), Value::I32(b)) => Ok((Value::I32(*a as i32), Value::I32(*b))),
            (Value::I32(a), Value::I8(b)) => Ok((Value::I32(*a), Value::I32(*b as i32))),
            (Value::I16(a), Value::I32(b)) => Ok((Value::I32(*a as i32), Value::I32(*b))),
            (Value::I32(a), Value::I16(b)) => Ok((Value::I32(*a), Value::I32(*b as i32))),
            
            // Float promotions
            (Value::F32(a), Value::F64(b)) => Ok((Value::F64(*a as f64), Value::F64(*b))),
            (Value::F64(a), Value::F32(b)) => Ok((Value::F64(*a), Value::F64(*b as f64))),
            (Value::I32(a), Value::F32(b)) => Ok((Value::F32(*a as f32), Value::F32(*b))),
            (Value::F32(a), Value::I32(b)) => Ok((Value::F32(*a), Value::F32(*b as f32))),
            
            // Default: no promotion
            _ => Ok((left.clone(), right.clone())),
        }
    }

    fn eval_unary_op(&self, op: &UnaryOperator, val: &Value) -> Result<Value> {
        match op {
            UnaryOperator::Minus => impl_unary_op!(val, -, signed_only),
            UnaryOperator::Not => match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(RuntimeError {
                    message: "Unary ! only supports boolean values".into(),
                }),
            },
        }
    }

    fn cast_value(&self, val: &Value, target_type: &Type) -> Result<Value> {
        // First try the built-in cast_to method
        if let Some(casted) = val.cast_to(target_type) {
            return Ok(casted);
        }
        
        // Use our macro for comprehensive casting
        impl_all_upcasts!(val, target_type)
    }
}

trait Truthy {
    fn is_truthy(&self) -> bool;
}

impl Truthy for Value {
    fn is_truthy(&self) -> bool {
        is_truthy!(self)
    }
}