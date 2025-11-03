use std::collections::HashMap;

pub struct BuiltinRegistry {
    functions: HashMap<String, fn(Vec<Value>) -> Result<Value, RuntimeError>>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        let mut registry = BuiltinRegistry {
            functions: HashMap::new(),
        };
        
        registry.register("to_string", to_string);
        registry.register("count_len", count_len);
        
        registry
    }
    
    pub fn register(&mut self, name: &str, func: fn(Vec<Value>) -> Result<Value, RuntimeError>) {
        self.functions.insert(name.to_string(), func);
    }
    
    pub fn call(&self, name: &str, args: Vec<Value>) -> Option<Result<Value, RuntimeError>> {
        self.functions.get(name).map(|f| f(args))
    }
    
    pub fn exists(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}


use core::result::Result::{self, Err, Ok};

use crate::compiler::{ast::{self, Value}, interpreter::RuntimeError};


pub fn to_string(args: Vec<ast::Value>) -> Result<Value, RuntimeError> {
    
    if args.len() != 1 {
        return Err(RuntimeError { message: "to_string expects one argument".to_string() });
    }

    // let v = &args[0];

    // match v {
    //     Value::I8(i) => Ok(Value::Str(i.to_string())),
    //     Value::I16(i) => Ok(Value::Str(i.to_string())),
    //     Value::I32(i) => Ok(Value::Str(i.to_string())),
    //     Value::I64(i) => Ok(Value::Str(i.to_string())),
    //     Value::I128(i) => Ok(Value::Str(i.to_string())),
    //     Value::U8(u) => Ok(Value::Str(u.to_string())),
    //     Value::U16(u) => Ok(Value::Str(u.to_string())),
    //     Value::U32(u) => Ok(Value::Str(u.to_string())),
    //     Value::U64(u) => Ok(Value::Str(u.to_string())),
    //     Value::U128(u) => Ok(Value::Str(u.to_string())),
    //     Value::F32(f) => Ok(Value::Str(f.to_string())),
    //     Value::F64(f) => Ok(Value::Str(f.to_string())),
    //     Value::Usize(u) => Ok(Value::Str(u.to_string())),
    //     Value::Bool(b) => Ok(Value::Str(b.to_string())),
    //     _ => Err(RuntimeError { message: "Cannot convert to string".to_string() })
    // }

    let v = &args[0];
    let s = match v {
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
                        _ => return Err(RuntimeError { message: "Cannot convert to string".to_string() }),
                        
                    };
    return Ok(Value::String(s));

}

pub fn count_len(args: Vec<ast::Value>) -> Result<Value, RuntimeError> {
    // match &args[0] {
    //     Value::I8(i) => Ok(Value::Usize(i.to_string().len())),
    //     Value::I16(i) => Ok(Value::Usize(i.to_string().len())),
    //     Value::I32(i) => Ok(Value::Usize(i.to_string().len())),
    //     Value::I64(i) => Ok(Value::Usize(i.to_string().len())),
    //     Value::I128(i) => Ok(Value::Usize(i.to_string().len())),
    //     Value::U8(u) => Ok(Value::Usize(u.to_string().len())),
    //     Value::U16(u) => Ok(Value::Usize(u.to_string().len())),
    //     Value::U32(u) => Ok(Value::Usize(u.to_string().len())),
    //     Value::U64(u) => Ok(Value::Usize(u.to_string().len())),
    //     Value::U128(u) => Ok(Value::Usize(u.to_string().len())),
    //     Value::F32(f) => Ok(Value::Usize(f.to_string().len())),
    //     Value::F64(f) => Ok(Value::Usize(f.to_string().len())),
    //     Value::Usize(u) => Ok(Value::Usize(u.to_string().len())),
    //     Value::Bool(b) => Ok(Value::Usize(b.to_string().len())),
    //     Value::Str(s) => Ok(Value::Usize(s.to_string().len())),
    //     _ => Err(RuntimeError { message: "Cannot convert to string".to_string() })
    // }

    let v = &args[0];
    let s = match v {
                        Value::I8(n) => n.to_string().len(),
                        Value::I16(n) => n.to_string().len(),
                        Value::I32(n) => n.to_string().len(),
                        Value::I64(n) => n.to_string().len(),
                        Value::I128(n) => n.to_string().len(),
                        Value::U8(n) => n.to_string().len(),
                        Value::U16(n) => n.to_string().len(),
                        Value::U32(n) => n.to_string().len(),
                        Value::U64(n) => n.to_string().len(),
                        Value::U128(n) => n.to_string().len(),
                        Value::F32(n) => n.to_string().len(),
                        Value::F64(n) => n.to_string().len(),
                        Value::Usize(n) => n.clone(),
                        Value::Bool(b) => b.to_string().len(),
                        Value::String(s) => s.len(),
                        _ => return Err(RuntimeError { message: "Cannot convert to string".to_string() }),
                        
                    };
    return Ok(Value::Usize(s));
}

