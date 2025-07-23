// use crate::compiler::ast::Type;



// #[derive(Clone, Debug, PartialEq)]
// pub enum Value {
//     I8(i8),
//     I16(i16),
//     I32(i32),
//     I64(i64),
//     I128(i128),
//     U8(u8),
//     U16(u16),
//     U32(u32),
//     U64(u64),
//     U128(u128),
//     F32(f32),
//     F64(f64),
//     Bool(bool),
//     Str(String),
// }

// impl Value {
//     pub fn to_string(&self) -> String {
//         match self {
//             Value::I8(n) => n.to_string(),
//             Value::I16(n) => n.to_string(),
//             Value::I32(n) => n.to_string(),
//             Value::I64(n) => n.to_string(),
//             Value::I128(n) => n.to_string(),
//             Value::U8(n) => n.to_string(),
//             Value::U16(n) => n.to_string(),
//             Value::U32(n) => n.to_string(),
//             Value::U64(n) => n.to_string(),
//             Value::U128(n) => n.to_string(),
//             Value::F32(n) => n.to_string(),
//             Value::F64(n) => n.to_string(),
//             Value::Bool(b) => b.to_string(),
//             Value::Str(s) => s.clone(),
//         }
//     }

//     pub fn matches_type(&self, typ: &Type) -> bool {
//         match (self, typ) {
//             (Value::I8(_), Type::I8) => true,
//             (Value::I16(_), Type::I16) => true,
//             (Value::I32(_), Type::I32) => true,
//             (Value::I64(_), Type::I64) => true,
//             (Value::I128(_), Type::I128) => true,
//             (Value::U8(_), Type::U8) => true,
//             (Value::U16(_), Type::U16) => true,
//             (Value::U32(_), Type::U32) => true,
//             (Value::U64(_), Type::U64) => true,
//             (Value::U128(_), Type::U128) => true,
//             (Value::F32(_), Type::F32) => true,
//             (Value::F64(_), Type::F64) => true,
//             (Value::Bool(_), Type::Bool) => true,
//             (Value::Str(_), Type::String) => true,
//             _ => false,
//         }
//     }

//     pub  fn cast_to(&self, target_type: &Type) -> Option<Value> {
//         match (self, target_type) {
//             (Value::I8(n), Type::I16) => Some(Value::I16(*n as i16)),
//             (Value::I8(n), Type::I32) => Some(Value::I32(*n as i32)),
//             (Value::I8(n), Type::I64) => Some(Value::I64(*n as i64)),
//             (Value::I8(n), Type::I128) => Some(Value::I128(*n as i128)),
//             (Value::U8(n), Type::U16) => Some(Value::U16(*n as u16)),
//             (Value::U8(n), Type::U32) => Some(Value::U32(*n as u32)),
//             (Value::U8(n), Type::U64) => Some(Value::U64(*n as u64)),
//             (Value::U8(n), Type::U128) => Some(Value::U128(*n as u128)),
//             (Value::F32(n), Type::F64) => Some(Value::F64(*n as f64)),
//             _ => None,
//         }
//     }
// }
