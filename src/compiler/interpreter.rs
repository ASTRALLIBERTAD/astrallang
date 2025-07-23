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

impl std::error::Error for RuntimeError {}

#[derive(Clone, Debug)]
pub struct FunctionDef {
    pub params: Vec<Param>,
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
    ) -> Result<Value, RuntimeError> {
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
        let mut local_env = HashMap::new();
        for (param, arg) in func.params.iter().zip(args.iter()) {
            // Convert argument to parameter type if needed
            let converted_arg = self.convert_value_to_param_type(arg, &param.param_type)?;
            local_env.insert(param.name.clone(), converted_arg);
        }

        // Execute function body
        match self.run_block(&func.body, &mut local_env)? {
            Some(val) => Ok(val),
            None => Ok(Value::I32(0)), // Default return value
        }
    }

    fn convert_value_to_param_type(&self, value: &Value, param_type: &ParamType) -> Result<Value, RuntimeError> {
        match param_type {
            ParamType::Number(target_type) => {
                if let Some(converted) = value.cast_to(target_type) {
                    Ok(converted)
                } else {
                    // Try manual conversion
                    self.cast_value(value, target_type)
                }
            }
            ParamType::String => {
                match value {
                    Value::Str(_) => Ok(value.clone()),
                    _ => Ok(Value::Str(value.to_string())),
                }
            }
            ParamType::Bool => {
                match value {
                    Value::Bool(_) => Ok(value.clone()),
                    _ => Ok(Value::Bool(value.is_truthy())),
                }
            }
            ParamType::Any => Ok(value.clone()),
        }
    }

    fn run_block(
        &mut self,
        stmts: &[Stmt],
        env: &mut HashMap<String, Value>,
    ) -> Result<Option<Value>, RuntimeError> {
        for stmt in stmts {
            match self.eval_stmt(stmt, env)? {
                Some(val) => return Ok(Some(val)), // Early return
                None => continue,
            }
        }
        Ok(None)
    }

    fn eval_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
    ) -> Result<Option<Value>, RuntimeError> {
        match stmt {
            Stmt::Let(name, type_hint, expr) => {
                let val = self.eval_expr(expr, env)?;
                
                // Convert value to specified type if not Any
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
                println!("{}", val.to_string());
                Ok(None)
            }
            Stmt::Return(Some(expr)) => {
                let val = self.eval_expr(expr, env)?;
                Ok(Some(val))
            }
            Stmt::Return(None) => Ok(Some(Value::I32(0))),
            Stmt::If(cond, then_body, else_body) => {
                if self.eval_expr(cond, env)?.is_truthy() {
                    self.run_block(then_body, env)
                } else if let Some(else_body) = else_body {
                    self.run_block(else_body, env)
                } else {
                    Ok(None)
                }
            }
            Stmt::While(cond, body) => {
                while self.eval_expr(cond, env)?.is_truthy() {
                    if let Some(val) = self.run_block(body, env)? {
                        return Ok(Some(val)); // Handle return from within loop
                    }
                }
                Ok(None)
            }
            Stmt::Block(stmts) => {
                // Create new scope for block
                let mut block_env = env.clone();
                self.run_block(stmts, &mut block_env)
            }
            Stmt::Expression(expr) => {
                self.eval_expr(expr, env)?;
                Ok(None)
            }
            Stmt::Function(_) => Ok(None), // Function definitions are handled in first pass
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(val) => Ok(val.clone()),
            Expr::Ident(name) => {
                // Check local environment first, then globals
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
                // Get function definition
                let func = self
                    .functions
                    .get(name)
                    .ok_or_else(|| RuntimeError {
                        message: format!("Function '{}' not found", name),
                    })?
                    .clone();

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg, env)?);
                }

                // Create a mutable copy of env for the function call
                let mut temp_env = env.clone();
                self.call_function(&func, arg_values, &mut temp_env)
            }
        }
    }

    fn eval_binary_op(
        &self,
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> Result<Value, RuntimeError> {
        // Try to promote types to a common type for arithmetic operations
        let (promoted_left, promoted_right) = self.promote_numeric_types(left, right)?;
        
        match (&promoted_left, &promoted_right) {
            // I8 operations
            (Value::I8(a), Value::I8(b)) => Ok(match op {
                BinaryOperator::Add => Value::I8(a.wrapping_add(*b)),
                BinaryOperator::Subtract => Value::I8(a.wrapping_sub(*b)),
                BinaryOperator::Multiply => Value::I8(a.wrapping_mul(*b)),
                BinaryOperator::Divide => {
                    if *b == 0 { return Err(RuntimeError { message: "Division by zero".into() }); }
                    Value::I8(a / b)
                }
                BinaryOperator::Modulo => {
                    if *b == 0 { return Err(RuntimeError { message: "Modulo by zero".into() }); }
                    Value::I8(a % b)
                }
                BinaryOperator::Equal => Value::Bool(a == b),
                BinaryOperator::NotEqual => Value::Bool(a != b),
                BinaryOperator::Less => Value::Bool(a < b),
                BinaryOperator::Greater => Value::Bool(a > b),
                BinaryOperator::LessEqual => Value::Bool(a <= b),
                BinaryOperator::GreaterEqual => Value::Bool(a >= b),
            }),
            
            // I16 operations
            (Value::I16(a), Value::I16(b)) => Ok(match op {
                BinaryOperator::Add => Value::I16(a.wrapping_add(*b)),
                BinaryOperator::Subtract => Value::I16(a.wrapping_sub(*b)),
                BinaryOperator::Multiply => Value::I16(a.wrapping_mul(*b)),
                BinaryOperator::Divide => {
                    if *b == 0 { return Err(RuntimeError { message: "Division by zero".into() }); }
                    Value::I16(a / b)
                }
                BinaryOperator::Modulo => {
                    if *b == 0 { return Err(RuntimeError { message: "Modulo by zero".into() }); }
                    Value::I16(a % b)
                }
                BinaryOperator::Equal => Value::Bool(a == b),
                BinaryOperator::NotEqual => Value::Bool(a != b),
                BinaryOperator::Less => Value::Bool(a < b),
                BinaryOperator::Greater => Value::Bool(a > b),
                BinaryOperator::LessEqual => Value::Bool(a <= b),
                BinaryOperator::GreaterEqual => Value::Bool(a >= b),
            }),

            // I32 operations
            (Value::I32(a), Value::I32(b)) => Ok(match op {
                BinaryOperator::Add => Value::I32(a.wrapping_add(*b)),
                BinaryOperator::Subtract => Value::I32(a.wrapping_sub(*b)),
                BinaryOperator::Multiply => Value::I32(a.wrapping_mul(*b)),
                BinaryOperator::Divide => {
                    if *b == 0 { return Err(RuntimeError { message: "Division by zero".into() }); }
                    Value::I32(a / b)
                }
                BinaryOperator::Modulo => {
                    if *b == 0 { return Err(RuntimeError { message: "Modulo by zero".into() }); }
                    Value::I32(a % b)
                }
                BinaryOperator::Equal => Value::Bool(a == b),
                BinaryOperator::NotEqual => Value::Bool(a != b),
                BinaryOperator::Less => Value::Bool(a < b),
                BinaryOperator::Greater => Value::Bool(a > b),
                BinaryOperator::LessEqual => Value::Bool(a <= b),
                BinaryOperator::GreaterEqual => Value::Bool(a >= b),
            }),

            // I64 operations
            (Value::I64(a), Value::I64(b)) => Ok(match op {
                BinaryOperator::Add => Value::I64(a.wrapping_add(*b)),
                BinaryOperator::Subtract => Value::I64(a.wrapping_sub(*b)),
                BinaryOperator::Multiply => Value::I64(a.wrapping_mul(*b)),
                BinaryOperator::Divide => {
                    if *b == 0 { return Err(RuntimeError { message: "Division by zero".into() }); }
                    Value::I64(a / b)
                }
                BinaryOperator::Modulo => {
                    if *b == 0 { return Err(RuntimeError { message: "Modulo by zero".into() }); }
                    Value::I64(a % b)
                }
                BinaryOperator::Equal => Value::Bool(a == b),
                BinaryOperator::NotEqual => Value::Bool(a != b),
                BinaryOperator::Less => Value::Bool(a < b),
                BinaryOperator::Greater => Value::Bool(a > b),
                BinaryOperator::LessEqual => Value::Bool(a <= b),
                BinaryOperator::GreaterEqual => Value::Bool(a >= b),
            }),

            // F32 operations
            (Value::F32(a), Value::F32(b)) => Ok(match op {
                BinaryOperator::Add => Value::F32(a + b),
                BinaryOperator::Subtract => Value::F32(a - b),
                BinaryOperator::Multiply => Value::F32(a * b),
                BinaryOperator::Divide => {
                    if *b == 0.0 { return Err(RuntimeError { message: "Division by zero".into() }); }
                    Value::F32(a / b)
                }
                BinaryOperator::Modulo => {
                    if *b == 0.0 { return Err(RuntimeError { message: "Modulo by zero".into() }); }
                    Value::F32(a % b)
                }
                BinaryOperator::Equal => Value::Bool((a - b).abs() < f32::EPSILON),
                BinaryOperator::NotEqual => Value::Bool((a - b).abs() >= f32::EPSILON),
                BinaryOperator::Less => Value::Bool(a < b),
                BinaryOperator::Greater => Value::Bool(a > b),
                BinaryOperator::LessEqual => Value::Bool(a <= b),
                BinaryOperator::GreaterEqual => Value::Bool(a >= b),
            }),

            // F64 operations
            (Value::F64(a), Value::F64(b)) => Ok(match op {
                BinaryOperator::Add => Value::F64(a + b),
                BinaryOperator::Subtract => Value::F64(a - b),
                BinaryOperator::Multiply => Value::F64(a * b),
                BinaryOperator::Divide => {
                    if *b == 0.0 { return Err(RuntimeError { message: "Division by zero".into() }); }
                    Value::F64(a / b)
                }
                BinaryOperator::Modulo => {
                    if *b == 0.0 { return Err(RuntimeError { message: "Modulo by zero".into() }); }
                    Value::F64(a % b)
                }
                BinaryOperator::Equal => Value::Bool((a - b).abs() < f64::EPSILON),
                BinaryOperator::NotEqual => Value::Bool((a - b).abs() >= f64::EPSILON),
                BinaryOperator::Less => Value::Bool(a < b),
                BinaryOperator::Greater => Value::Bool(a > b),
                BinaryOperator::LessEqual => Value::Bool(a <= b),
                BinaryOperator::GreaterEqual => Value::Bool(a >= b),
            }),

            // Boolean operations
            (Value::Bool(a), Value::Bool(b)) => Ok(match op {
                BinaryOperator::Equal => Value::Bool(a == b),
                BinaryOperator::NotEqual => Value::Bool(a != b),
                _ => return Err(RuntimeError {
                    message: format!("Unsupported operation {:?} for boolean values", op),
                }),
            }),

            // String operations
            (Value::Str(a), Value::Str(b)) => Ok(match op {
                BinaryOperator::Add => Value::Str(format!("{}{}", a, b)),
                BinaryOperator::Equal => Value::Bool(a == b),
                BinaryOperator::NotEqual => Value::Bool(a != b),
                BinaryOperator::Less => Value::Bool(a < b),
                BinaryOperator::Greater => Value::Bool(a > b),
                BinaryOperator::LessEqual => Value::Bool(a <= b),
                BinaryOperator::GreaterEqual => Value::Bool(a >= b),
                _ => return Err(RuntimeError {
                    message: format!("Unsupported operation {:?} for string values", op),
                }),
            }),

            _ => Err(RuntimeError {
                message: format!("Type mismatch in binary operation: {:?} {:?} {:?}", promoted_left, op, promoted_right),
            }),
        }
    }

    fn promote_numeric_types(&self, left: &Value, right: &Value) -> Result<(Value, Value), RuntimeError> {
        // Implement type promotion rules
        match (left, right) {
            // Same types - no promotion needed
            (Value::I8(_), Value::I8(_)) |
            (Value::I16(_), Value::I16(_)) |
            (Value::I32(_), Value::I32(_)) |
            (Value::I64(_), Value::I64(_)) |
            (Value::F32(_), Value::F32(_)) |
            (Value::F64(_), Value::F64(_)) => Ok((left.clone(), right.clone())),
            
            // Integer promotions to larger types
            (Value::U8(a), Value::U8(b)) => Ok((Value::U8(*a), Value::U8(*b))),
            (Value::U8(a), Value::U16(b)) => Ok((Value::U16(*a as u16), Value::U16(*b))),
            (Value::U16(a), Value::U8(b)) => Ok((Value::U16(*a), Value::U16(*b as u16))),
            
            // Mixed integer types - promote to largest
            (Value::I8(a), Value::I16(b)) => Ok((Value::I16(*a as i16), Value::I16(*b))),
            (Value::I16(a), Value::I8(b)) => Ok((Value::I16(*a), Value::I16(*b as i16))),
            (Value::I8(a), Value::I32(b)) => Ok((Value::I32(*a as i32), Value::I32(*b))),
            (Value::I32(a), Value::I8(b)) => Ok((Value::I32(*a), Value::I32(*b as i32))),
            (Value::I16(a), Value::I32(b)) => Ok((Value::I32(*a as i32), Value::I32(*b))),
            (Value::I32(a), Value::I16(b)) => Ok((Value::I32(*a), Value::I32(*b as i32))),
            
            // Unsigned to signed promotions
            (Value::U8(a), Value::I16(b)) => Ok((Value::I16(*a as i16), Value::I16(*b))),
            (Value::I16(a), Value::U8(b)) => Ok((Value::I16(*a), Value::I16(*b as i16))),
            (Value::U8(a), Value::I32(b)) => Ok((Value::I32(*a as i32), Value::I32(*b))),
            (Value::I32(a), Value::U8(b)) => Ok((Value::I32(*a), Value::I32(*b as i32))),
            
            // Float promotions
            (Value::I8(a), Value::F32(b)) => Ok((Value::F32(*a as f32), Value::F32(*b))),
            (Value::F32(a), Value::I8(b)) => Ok((Value::F32(*a), Value::F32(*b as f32))),
            (Value::I16(a), Value::F32(b)) => Ok((Value::F32(*a as f32), Value::F32(*b))),
            (Value::F32(a), Value::I16(b)) => Ok((Value::F32(*a), Value::F32(*b as f32))),
            (Value::I32(a), Value::F32(b)) => Ok((Value::F32(*a as f32), Value::F32(*b))),
            (Value::F32(a), Value::I32(b)) => Ok((Value::F32(*a), Value::F32(*b as f32))),
            
            (Value::F32(a), Value::F64(b)) => Ok((Value::F64(*a as f64), Value::F64(*b))),
            (Value::F64(a), Value::F32(b)) => Ok((Value::F64(*a), Value::F64(*b as f64))),
            
            // Non-numeric types or incompatible combinations
            _ => Ok((left.clone(), right.clone())),
        }
    }

    fn eval_unary_op(&self, op: &UnaryOperator, val: &Value) -> Result<Value, RuntimeError> {
        match op {
            UnaryOperator::Minus => match val {
                Value::I8(n) => Ok(Value::I8(-n)),
                Value::I16(n) => Ok(Value::I16(-n)),
                Value::I32(n) => Ok(Value::I32(-n)),
                Value::I64(n) => Ok(Value::I64(-n)),
                Value::I128(n) => Ok(Value::I128(-n)),
                Value::F32(n) => Ok(Value::F32(-n)),
                Value::F64(n) => Ok(Value::F64(-n)),
                // Handle unsigned types by converting to signed
                Value::U8(n) => Ok(Value::I16(-(*n as i16))),
                Value::U16(n) => Ok(Value::I32(-(*n as i32))),
                Value::U32(n) => Ok(Value::I64(-(*n as i64))),
                Value::U64(n) => Ok(Value::I128(-(*n as i128))),
                _ => Err(RuntimeError {
                    message: format!("Unary minus not supported for type {:?}", val),
                }),
            },
            UnaryOperator::Not => match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(RuntimeError {
                    message: "Unary ! only supports boolean values".into(),
                }),
            },
        }
    }

    fn cast_value(&self, val: &Value, target_type: &Type) -> Result<Value, RuntimeError> {
        // First try the built-in cast_to method from Value
        if let Some(casted) = val.cast_to(target_type) {
            return Ok(casted);
        }
        
        // Handle additional casting cases
        match (val, target_type) {
            // Integer conversions (potentially lossy)
            (Value::I16(n), Type::I8) => Ok(Value::I8(*n as i8)),
            (Value::I32(n), Type::I8) => Ok(Value::I8(*n as i8)),
            (Value::I32(n), Type::I16) => Ok(Value::I16(*n as i16)),
            (Value::I64(n), Type::I32) => Ok(Value::I32(*n as i32)),
            (Value::I128(n), Type::I64) => Ok(Value::I64(*n as i64)),
            
            // Unsigned integer conversions
            (Value::U16(n), Type::U8) => Ok(Value::U8(*n as u8)),
            (Value::U32(n), Type::U8) => Ok(Value::U8(*n as u8)),
            (Value::U32(n), Type::U16) => Ok(Value::U16(*n as u16)),
            (Value::U64(n), Type::U32) => Ok(Value::U32(*n as u32)),
            (Value::U128(n), Type::U64) => Ok(Value::U64(*n as u64)),
            
            // Unsigned to signed conversions
            (Value::U8(n), Type::I8) => Ok(Value::I8(*n as i8)),
            (Value::U8(n), Type::I16) => Ok(Value::I16(*n as i16)),
            (Value::U8(n), Type::I32) => Ok(Value::I32(*n as i32)),
            (Value::U16(n), Type::I16) => Ok(Value::I16(*n as i16)),
            (Value::U16(n), Type::I32) => Ok(Value::I32(*n as i32)),
            (Value::U32(n), Type::I32) => Ok(Value::I32(*n as i32)),
            
            // Float conversions
            (Value::F64(n), Type::F32) => Ok(Value::F32(*n as f32)),
            (Value::F32(n), Type::I32) => Ok(Value::I32(*n as i32)),
            (Value::F64(n), Type::I32) => Ok(Value::I32(*n as i32)),
            (Value::I32(n), Type::F32) => Ok(Value::F32(*n as f32)),
            (Value::I32(n), Type::F64) => Ok(Value::F64(*n as f64)),
            (Value::U8(n), Type::F32) => Ok(Value::F32(*n as f32)),
            (Value::U16(n), Type::F32) => Ok(Value::F32(*n as f32)),
            
            // Boolean conversions
            (Value::Bool(b), Type::I32) => Ok(Value::I32(if *b { 1 } else { 0 })),
            (Value::I32(n), Type::Bool) => Ok(Value::Bool(*n != 0)),
            
            // String conversions
            (val, Type::String) => Ok(Value::Str(val.to_string())),
            
            _ => Err(RuntimeError {
                message: format!("Cannot cast {:?} to {:?}", val, target_type),
            }),
        }
    }

    fn value_to_string(&self, val: &Value) -> String {
        val.to_string()
    }
}

trait Truthy {
    fn is_truthy(&self) -> bool;
}

impl Truthy for Value {
    fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::I8(n) => *n != 0,
            Value::I16(n) => *n != 0,
            Value::I32(n) => *n != 0,
            Value::I64(n) => *n != 0,
            Value::I128(n) => *n != 0,
            Value::U8(n) => *n != 0,
            Value::U16(n) => *n != 0,
            Value::U32(n) => *n != 0,
            Value::U64(n) => *n != 0,
            Value::U128(n) => *n != 0,
            Value::F32(n) => *n != 0.0,
            Value::F64(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
        }
    }
}