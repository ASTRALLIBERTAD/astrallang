// src/compiler/macros.rs
// This file contains all the macros to reduce code repetition

/// Macro to implement binary arithmetic operations for all numeric types
macro_rules! impl_arithmetic_op {
    ($left:expr, $right:expr, $op:tt, wrapping) => {
        match ($left, $right) {
            (Value::I8(a), Value::I8(b)) => Ok(Value::I8(a.$op(*b))),
            (Value::I16(a), Value::I16(b)) => Ok(Value::I16(a.$op(*b))),
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a.$op(*b))),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.$op(*b))),
            (Value::I128(a), Value::I128(b)) => Ok(Value::I128(a.$op(*b))),
            (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a.$op(*b))),
            (Value::U16(a), Value::U16(b)) => Ok(Value::U16(a.$op(*b))),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a.$op(*b))),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a.$op(*b))),
            (Value::U128(a), Value::U128(b)) => Ok(Value::U128(a.$op(*b))),
            _ => Err(RuntimeError {
                message: format!("Cannot perform operation on {:?} and {:?}", $left, $right),
            }),
        }
    };
    ($left:expr, $right:expr, $op:tt) => {
        match ($left, $right) {
            (Value::I8(a), Value::I8(b)) => Ok(Value::I8(a $op b)),
            (Value::I16(a), Value::I16(b)) => Ok(Value::I16(a $op b)),
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a $op b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a $op b)),
            (Value::I128(a), Value::I128(b)) => Ok(Value::I128(a $op b)),
            (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a $op b)),
            (Value::U16(a), Value::U16(b)) => Ok(Value::U16(a $op b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a $op b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a $op b)),
            (Value::U128(a), Value::U128(b)) => Ok(Value::U128(a $op b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a $op b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a $op b)),
            _ => Err(RuntimeError {
                message: format!("Cannot perform operation on {:?} and {:?}", $left, $right),
            }),
        }
    };
}

/// Macro for comparison operations
macro_rules! impl_comparison_op {
    ($left:expr, $right:expr, $op:tt) => {
        match ($left, $right) {
            (Value::I8(a), Value::I8(b)) => Ok(Value::Bool(a $op b)),
            (Value::I16(a), Value::I16(b)) => Ok(Value::Bool(a $op b)),
            (Value::I32(a), Value::I32(b)) => Ok(Value::Bool(a $op b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a $op b)),
            (Value::I128(a), Value::I128(b)) => Ok(Value::Bool(a $op b)),
            (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a $op b)),
            (Value::U16(a), Value::U16(b)) => Ok(Value::Bool(a $op b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::Bool(a $op b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::Bool(a $op b)),
            (Value::U128(a), Value::U128(b)) => Ok(Value::Bool(a $op b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::Bool(a $op b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a $op b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a $op b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a $op b)),
            _ => Err(RuntimeError {
                message: format!("Cannot compare {:?} and {:?}", $left, $right),
            }),
        }
    };
}

/// Macro to implement unary operations
macro_rules! impl_unary_op {
    ($val:expr, $op:tt, signed_only) => {
        match $val {
            Value::I8(n) => Ok(Value::I8($op n)),
            Value::I16(n) => Ok(Value::I16($op n)),
            Value::I32(n) => Ok(Value::I32($op n)),
            Value::I64(n) => Ok(Value::I64($op n)),
            Value::I128(n) => Ok(Value::I128($op n)),
            Value::F32(n) => Ok(Value::F32($op n)),
            Value::F64(n) => Ok(Value::F64($op n)),
            // Convert unsigned to signed for negation
            Value::U8(n) => Ok(Value::I16($op (*n as i16))),
            Value::U16(n) => Ok(Value::I32($op (*n as i32))),
            Value::U32(n) => Ok(Value::I64($op (*n as i64))),
            Value::U64(n) => Ok(Value::I128($op (*n as i128))),
            _ => Err(RuntimeError {
                message: format!("Cannot apply unary operator to {:?}", $val),
            }),
        }
    };
}

/// Macro to implement value conversions to string
macro_rules! value_to_string {
    ($val:expr) => {
        match $val {
            Value::I8(n) => n.to_string(),
            Value::I16(n) => n.to_string(),
            Value::I32(n) => n.to_string(),
            Value::I64(n) => n.to_string(),
            Value::I128(n) => n.to_string(),
            Value::U8(n) => n.to_string(),
            Value::U16(n) => n.to_string(),
            Value::U32(n) => n.to_string(),
            Value::U64(n) => n.to_string(),
            Value::U128(n) => n.to_string(),
            Value::F32(n) => n.to_string(),
            Value::F64(n) => n.to_string(),
            Value::Usize(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Str(s) => s.to_string(),
        }
    };
}

/// Macro to get the length/size of a value
macro_rules! value_length {
    ($val:expr) => {
        match $val {
            Value::String(s) => s.len(),
            Value::Str(s) => s.len(),
            Value::Usize(n) => *n,
            v => value_to_string!(v).len(),
        }
    };
}

/// Macro to check if a value is truthy
macro_rules! is_truthy {
    ($val:expr) => {
        match $val {
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
            Value::Usize(n) => *n != 0,
            Value::Str(s) => !s.is_empty(),
            Value::String(s) => !s.is_empty(),
        }
    };
}

/// Macro to implement type casting with less repetition
macro_rules! impl_cast {
    ($val:expr, $from:ident => $to:ident as $cast_type:ty) => {
        if let Value::$from(n) = $val {
            return Ok(Value::$to(*n as $cast_type));
        }
    };
}

/// Macro to generate all upcast rules (safe casts to larger types)
macro_rules! impl_all_upcasts {
    ($val:expr, $target:expr) => {
        match ($val, $target) {
            // Signed integer upcasts
            (Value::I8(n), Type::I16) => Ok(Value::I16(*n as i16)),
            (Value::I8(n), Type::I32) => Ok(Value::I32(*n as i32)),
            (Value::I8(n), Type::I64) => Ok(Value::I64(*n as i64)),
            (Value::I8(n), Type::I128) => Ok(Value::I128(*n as i128)),
            (Value::I16(n), Type::I32) => Ok(Value::I32(*n as i32)),
            (Value::I16(n), Type::I64) => Ok(Value::I64(*n as i64)),
            (Value::I16(n), Type::I128) => Ok(Value::I128(*n as i128)),
            (Value::I32(n), Type::I64) => Ok(Value::I64(*n as i64)),
            (Value::I32(n), Type::I128) => Ok(Value::I128(*n as i128)),
            (Value::I64(n), Type::I128) => Ok(Value::I128(*n as i128)),
            
            // Unsigned integer upcasts
            (Value::U8(n), Type::U16) => Ok(Value::U16(*n as u16)),
            (Value::U8(n), Type::U32) => Ok(Value::U32(*n as u32)),
            (Value::U8(n), Type::U64) => Ok(Value::U64(*n as u64)),
            (Value::U8(n), Type::U128) => Ok(Value::U128(*n as u128)),
            (Value::U16(n), Type::U32) => Ok(Value::U32(*n as u32)),
            (Value::U16(n), Type::U64) => Ok(Value::U64(*n as u64)),
            (Value::U16(n), Type::U128) => Ok(Value::U128(*n as u128)),
            (Value::U32(n), Type::U64) => Ok(Value::U64(*n as u64)),
            (Value::U32(n), Type::U128) => Ok(Value::U128(*n as u128)),
            (Value::U64(n), Type::U128) => Ok(Value::U128(*n as u128)),
            
            // Float upcasts
            (Value::F32(n), Type::F64) => Ok(Value::F64(*n as f64)),
            
            // Integer to float (potentially lossy for large integers)
            (Value::I8(n), Type::F32) => Ok(Value::F32(*n as f32)),
            (Value::I8(n), Type::F64) => Ok(Value::F64(*n as f64)),
            (Value::I16(n), Type::F32) => Ok(Value::F32(*n as f32)),
            (Value::I16(n), Type::F64) => Ok(Value::F64(*n as f64)),
            (Value::I32(n), Type::F32) => Ok(Value::F32(*n as f32)),
            (Value::I32(n), Type::F64) => Ok(Value::F64(*n as f64)),
            
            // String conversions
            (Value::Str(s), Type::String) => Ok(Value::String(s.to_string())),
            
            // Any other case
            _ => Err(RuntimeError {
                message: format!("Cannot cast {:?} to {:?}", $val.get_type(), $target),
            }),
        }
    };
}

// Export macros
pub(crate) use impl_arithmetic_op;
pub(crate) use impl_comparison_op;
pub(crate) use impl_unary_op;
pub(crate) use value_to_string;
pub(crate) use value_length;
pub(crate) use is_truthy;
pub(crate) use impl_cast;
pub(crate) use impl_all_upcasts;