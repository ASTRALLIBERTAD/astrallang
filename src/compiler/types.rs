use crate::compiler::ast::{Type, Value};

/// Type checking and conversion utilities
pub struct TypeSystem;

impl TypeSystem {
    /// Check if a value matches a type
    pub fn matches(value: &Value, typ: &Type) -> bool {
        match (value, typ) {
            (Value::I8(_), Type::I8) => true,
            (Value::I16(_), Type::I16) => true,
            (Value::I32(_), Type::I32) => true,
            (Value::I64(_), Type::I64) => true,
            (Value::I128(_), Type::I128) => true,
            (Value::U8(_), Type::U8) => true,
            (Value::U16(_), Type::U16) => true,
            (Value::U32(_), Type::U32) => true,
            (Value::U64(_), Type::U64) => true,
            (Value::U128(_), Type::U128) => true,
            (Value::F32(_), Type::F32) => true,
            (Value::F64(_), Type::F64) => true,
            (Value::Bool(_), Type::Bool) => true,
            (Value::Str(_), Type::Str) => true,
            (Value::String(_), Type::String) => true,
            (Value::Usize(_), Type::Usize) => true,
            (_, Type::Any) => true,
            _ => false,
        }
    }

    /// Check if one type can be safely cast to another
    pub fn is_safe_cast(from: &Type, to: &Type) -> bool {
        if from == to {
            return true;
        }

        matches!(
            (from, to),
            // Integer widening
            (Type::I8, Type::I16 | Type::I32 | Type::I64 | Type::I128) |
            (Type::I16, Type::I32 | Type::I64 | Type::I128) |
            (Type::I32, Type::I64 | Type::I128) |
            (Type::I64, Type::I128) |
            
            // Unsigned widening
            (Type::U8, Type::U16 | Type::U32 | Type::U64 | Type::U128) |
            (Type::U16, Type::U32 | Type::U64 | Type::U128) |
            (Type::U32, Type::U64 | Type::U128) |
            (Type::U64, Type::U128) |
            
            // Float widening
            (Type::F32, Type::F64) |
            
            // String conversions
            (Type::Str, Type::String)
        )
    }

    /// Get the "larger" type for binary operations
    pub fn common_type(left: &Type, right: &Type) -> Option<Type> {
        if left == right {
            return Some(left.clone());
        }

        match (left, right) {
            // Integer promotion
            (Type::I8, Type::I16) | (Type::I16, Type::I8) => Some(Type::I16),
            (Type::I8, Type::I32) | (Type::I32, Type::I8) => Some(Type::I32),
            (Type::I16, Type::I32) | (Type::I32, Type::I16) => Some(Type::I32),
            (Type::I32, Type::I64) | (Type::I64, Type::I32) => Some(Type::I64),
            
            // Float promotion
            (Type::F32, Type::F64) | (Type::F64, Type::F32) => Some(Type::F64),
            
            // Integer to float
            (Type::I32, Type::F32) | (Type::F32, Type::I32) => Some(Type::F32),
            (Type::I32, Type::F64) | (Type::F64, Type::I32) => Some(Type::F64),
            
            _ => None,
        }
    }
}
