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

use crate::compiler::{ast::{self, Value}, interpreter::RuntimeError, macros::{impl_unary_op, value_length, value_to_string}};


fn to_string(args: Vec<ast::Value>) -> Result<Value, RuntimeError> {
    
    if args.len() != 1 {
        return Err(RuntimeError { message: "to_string expects one argument".to_string() });
    }

    return Ok(Value::String(value_to_string!(&args[0])));

}

fn count_len(args: Vec<ast::Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError { message: "count_len expects one argument".to_string() });
    }
    return Ok(Value::Usize(value_length!(&args[0])));
}

// fn negate(args: Vec<ast::Value>) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError { message: "negate expects one argument".to_string() });
//     }

//     return Ok(impl_unary_op!(args[0], -, signed_only));


// }


