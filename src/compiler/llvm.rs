// use crate::compiler::ast::*;
// use std::collections::HashMap;
// use std::fmt::Write;

// pub struct LLVMCodegen {
//     output: String,
//     next_label: u32,
//     next_temp: u32,
//     functions: HashMap<String, FunctionInfo>,
// }

// #[derive(Clone)]
// struct FunctionInfo {
//     params: Vec<Param>,
//     return_type: String,
// }

// impl LLVMCodegen {
//     pub fn new() -> Self {
//         Self {
//             output: String::new(),
//             next_label: 0,
//             next_temp: 0,
//             functions: HashMap::new(),
//         }
//     }

//     pub fn generate(&mut self, stmts: &[Stmt]) -> Result<String, String> {
//         // Generate LLVM header and declarations
//         self.generate_header();

//         // First pass: collect function signatures
//         for stmt in stmts {
//             if let Stmt::Function(func) = stmt {
//                 self.functions.insert(
//                     func.name.clone(),
//                     FunctionInfo {
//                         params: func.params.clone(),
//                         return_type: self.type_to_llvm(&Type::I64), // Default return type
//                     }
//                 );
//             }
//         }

//         // Second pass: generate function definitions
//         for stmt in stmts {
//             if let Stmt::Function(func) = stmt {
//                 self.generate_function(func)?;
//             }
//         }

//         Ok(self.output.clone())
//     }

//     fn generate_header(&mut self) {
//         writeln!(self.output, "; LLVM IR generated from custom language").unwrap();
//         writeln!(self.output, "target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"").unwrap();
//         writeln!(self.output, "target triple = \"x86_64-unknown-linux-gnu\"").unwrap();
//         writeln!(self.output).unwrap();

//         // Declare external functions
//         writeln!(self.output, "declare i32 @printf(ptr, ...)").unwrap();
//         writeln!(self.output, "declare ptr @malloc(i64)").unwrap();
//         writeln!(self.output, "declare void @free(ptr)").unwrap();
//         writeln!(self.output).unwrap();

//         // Global string constants for printing
//         writeln!(self.output, "@.str.i8 = private unnamed_addr constant [5 x i8] c\"%hhd\\0A\\00\", align 1").unwrap();
//         writeln!(self.output, "@.str.i16 = private unnamed_addr constant [5 x i8] c\"%hd\\0A\\00\", align 1").unwrap();
//         writeln!(self.output, "@.str.i32 = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\", align 1").unwrap();
//         writeln!(self.output, "@.str.i64 = private unnamed_addr constant [5 x i8] c\"%ld\\0A\\00\", align 1").unwrap();
//         writeln!(self.output, "@.str.f32 = private unnamed_addr constant [4 x i8] c\"%f\\0A\\00\", align 1").unwrap();
//         writeln!(self.output, "@.str.f64 = private unnamed_addr constant [5 x i8] c\"%lf\\0A\\00\", align 1").unwrap();
//         writeln!(self.output, "@.str.bool_true = private unnamed_addr constant [6 x i8] c\"true\\0A\\00\", align 1").unwrap();
//         writeln!(self.output, "@.str.bool_false = private unnamed_addr constant [7 x i8] c\"false\\0A\\00\", align 1").unwrap();
//         writeln!(self.output, "@.str.str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\", align 1").unwrap();
//         writeln!(self.output).unwrap();
//     }

//     fn generate_function(&mut self, func: &Function) -> Result<(), String> {
//         // Reset temp counter for each function
//         self.next_temp = 0;

//         // Generate function signature
//         let mut param_decl = String::new();
//         for (i, param) in func.params.iter().enumerate() {
//             if i > 0 {
//                 param_decl.push_str(", ");
//             }
//             let llvm_type = self.param_type_to_llvm(&param.param_type);
//             param_decl.push_str(&format!("{} %{}", llvm_type, i));
//         }

//         writeln!(
//             self.output,
//             "define i64 @{}({}) {{",
//             func.name,
//             param_decl
//         ).unwrap();

//         // Entry block
//         writeln!(self.output, "entry:").unwrap();

//         // Allocate space for parameters and convert them
//         let mut local_vars = HashMap::new();
//         for (i, param) in func.params.iter().enumerate() {
//             let param_reg = i as u32;
//             let alloca = self.next_temp();
//             let converted_reg = self.next_temp();
            
//             // Allocate space for the parameter as i64
//             writeln!(self.output, "  %{} = alloca i64, align 8", alloca).unwrap();
            
//             // Convert parameter to i64 if needed
//             let llvm_type = self.param_type_to_llvm(&param.param_type);
//             if llvm_type == "i64" {
//                 writeln!(self.output, "  store i64 %{}, ptr %{}, align 8", param_reg, alloca).unwrap();
//             } else {
//                 // Convert to i64 before storing
//                 self.convert_to_i64(param_reg, &param.param_type, converted_reg)?;
//                 writeln!(self.output, "  store i64 %{}, ptr %{}, align 8", converted_reg, alloca).unwrap();
//             }
            
//             local_vars.insert(param.name.clone(), alloca);
//         }

//         // Generate function body
//         let has_explicit_return = self.has_return(&func.body);
//         self.generate_block(&func.body, &mut local_vars)?;

//         // If no explicit return, return 0
//         if !has_explicit_return {
//             writeln!(self.output, "  ret i64 0").unwrap();
//         }

//         writeln!(self.output, "}}").unwrap();
//         writeln!(self.output).unwrap();

//         Ok(())
//     }

//     fn generate_block(&mut self, stmts: &[Stmt], local_vars: &mut HashMap<String, u32>) -> Result<Option<u32>, String> {
//         let mut returned = false;
        
//         for stmt in stmts {
//             if returned {
//                 break; // Don't generate unreachable code
//             }
            
//             match stmt {
//                 Stmt::Let(name, type_hint, expr) => {
//                     let val_reg = self.generate_expr(expr, local_vars)?;
//                     let alloca = self.next_temp();
//                     writeln!(self.output, "  %{} = alloca i64, align 8", alloca).unwrap();
                    
//                     // Cast value to the specified type if needed
//                     let final_reg = if *type_hint != Type::Any {
//                         let types = ParamType::Any;
//                         let cast_reg = self.next_temp();
//                         self.cast_i64_to_type(val_reg, type_hint, cast_reg)?;
//                         let back_to_i64 = self.next_temp();
//                         self.convert_to_i64(cast_reg, &types, back_to_i64)?;
//                         back_to_i64
//                     } else {
//                         val_reg
//                     };
                    
//                     writeln!(self.output, "  store i64 %{}, ptr %{}, align 8", final_reg, alloca).unwrap();
//                     local_vars.insert(name.clone(), alloca);
//                 }

//                 Stmt::Print(expr) => {
//                     let val_reg = self.generate_expr(expr, local_vars)?;
//                     let temp = self.next_temp();
                    
//                     // For now, treat everything as i64 for printing
//                     writeln!(
//                         self.output,
//                         "  %{} = call i32 (ptr, ...) @printf(ptr @.str.i64, i64 %{})",
//                         temp,
//                         val_reg
//                     ).unwrap();
//                 }

//                 Stmt::Return(Some(expr)) => {
//                     let val_reg = self.generate_expr(expr, local_vars)?;
//                     writeln!(self.output, "  ret i64 %{}", val_reg).unwrap();
//                     returned = true;
//                 }

//                 Stmt::Return(None) => {
//                     writeln!(self.output, "  ret i64 0").unwrap();
//                     returned = true;
//                 }

//                 Stmt::If(condition, then_body, else_body) => {
//                     let cond_reg = self.generate_expr(condition, local_vars)?;
//                     let then_label = self.next_label();
//                     let else_label = self.next_label();
//                     let end_label = self.next_label();

//                     // Convert condition to boolean
//                     let bool_reg = self.next_temp();
//                     writeln!(self.output, "  %{} = icmp ne i64 %{}, 0", bool_reg, cond_reg).unwrap();
//                     writeln!(self.output, "  br i1 %{}, label %{}, label %{}", bool_reg, then_label, else_label).unwrap();

//                     // Then block
//                     writeln!(self.output, "{}:", then_label).unwrap();
//                     let then_returned = self.generate_block(then_body, local_vars)?.is_some();
//                     if !then_returned {
//                         writeln!(self.output, "  br label %{}", end_label).unwrap();
//                     }

//                     // Else block
//                     writeln!(self.output, "{}:", else_label).unwrap();
//                     let else_returned = if let Some(else_stmts) = else_body {
//                         self.generate_block(else_stmts, local_vars)?.is_some()
//                     } else {
//                         false
//                     };
                    
//                     if !else_returned {
//                         writeln!(self.output, "  br label %{}", end_label).unwrap();
//                     }

//                     // End block (only if not both branches returned)
//                     if !then_returned || !else_returned {
//                         writeln!(self.output, "{}:", end_label).unwrap();
//                     }
//                 }

//                 Stmt::While(condition, body) => {
//                     let loop_label = self.next_label();
//                     let body_label = self.next_label();
//                     let end_label = self.next_label();

//                     writeln!(self.output, "  br label %{}", loop_label).unwrap();

//                     // Loop condition check
//                     writeln!(self.output, "{}:", loop_label).unwrap();
//                     let cond_reg = self.generate_expr(condition, local_vars)?;
//                     let bool_reg = self.next_temp();
//                     writeln!(self.output, "  %{} = icmp ne i64 %{}, 0", bool_reg, cond_reg).unwrap();
//                     writeln!(self.output, "  br i1 %{}, label %{}, label %{}", bool_reg, body_label, end_label).unwrap();

//                     // Loop body
//                     writeln!(self.output, "{}:", body_label).unwrap();
//                     self.generate_block(body, local_vars)?;
//                     writeln!(self.output, "  br label %{}", loop_label).unwrap();

//                     // End of loop
//                     writeln!(self.output, "{}:", end_label).unwrap();
//                 }

//                 Stmt::Block(stmts) => {
//                     // Create new scope for block
//                     let mut block_vars = local_vars.clone();
//                     if let Some(_) = self.generate_block(stmts, &mut block_vars)? {
//                         returned = true;
//                     }
//                 }

//                 Stmt::Expression(expr) => {
//                     self.generate_expr(expr, local_vars)?;
//                 }

//                 Stmt::Function(_) => {
//                     // Function definitions are handled separately
//                 }
//             }
//         }
        
//         Ok(if returned { Some(0) } else { None })
//     }

//     fn generate_expr(&mut self, expr: &Expr, local_vars: &HashMap<String, u32>) -> Result<u32, String> {
//         match expr {

//             Expr::Match(, )
//             Expr::Literal(val) => {
//                 let reg = self.next_temp();
//                 match val {
//                     Value::I8(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::I16(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::I32(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::I64(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, n).unwrap(),
//                     Value::I128(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::U8(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::U16(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::U32(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::U64(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::U128(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::F32(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::F64(n) => writeln!(self.output, "  %{} = add i64 0, {}", reg, *n as i64).unwrap(),
//                     Value::Bool(b) => {
//                         let val = if *b { 1 } else { 0 };
//                         writeln!(self.output, "  %{} = add i64 0, {}", reg, val).unwrap();
//                     }
//                     Value::Str(s) => {
//                         // For strings, we'll store the length as a simple representation
//                         writeln!(self.output, "  %{} = add i64 0, {}", reg, s.len()).unwrap();
//                     }
//                 }
//                 Ok(reg)
//             }

//             Expr::Ident(name) => {
//                 let alloca = local_vars.get(name)
//                     .ok_or_else(|| format!("Undefined variable: {}", name))?;
//                 let reg = self.next_temp();
//                 writeln!(self.output, "  %{} = load i64, ptr %{}, align 8", reg, alloca).unwrap();
//                 Ok(reg)
//             }

//             Expr::BinaryOp(lhs, op, rhs) => {
//                 let left_reg = self.generate_expr(lhs, local_vars)?;
//                 let right_reg = self.generate_expr(rhs, local_vars)?;

//                 match op {
//                     BinaryOperator::Add => {
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = add i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::Subtract => {
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = sub i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::Multiply => {
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = mul i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::Divide => {
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = sdiv i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::Modulo => {
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = srem i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::Equal => {
//                         let cmp_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = icmp eq i64 %{}, %{}", cmp_reg, left_reg, right_reg).unwrap();
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = zext i1 %{} to i64", result_reg, cmp_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::NotEqual => {
//                         let cmp_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = icmp ne i64 %{}, %{}", cmp_reg, left_reg, right_reg).unwrap();
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = zext i1 %{} to i64", result_reg, cmp_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::Less => {
//                         let cmp_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = icmp slt i64 %{}, %{}", cmp_reg, left_reg, right_reg).unwrap();
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = zext i1 %{} to i64", result_reg, cmp_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::Greater => {
//                         let cmp_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = icmp sgt i64 %{}, %{}", cmp_reg, left_reg, right_reg).unwrap();
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = zext i1 %{} to i64", result_reg, cmp_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::LessEqual => {
//                         let cmp_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = icmp sle i64 %{}, %{}", cmp_reg, left_reg, right_reg).unwrap();
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = zext i1 %{} to i64", result_reg, cmp_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     BinaryOperator::GreaterEqual => {
//                         let cmp_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = icmp sge i64 %{}, %{}", cmp_reg, left_reg, right_reg).unwrap();
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = zext i1 %{} to i64", result_reg, cmp_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                 }
//             }

//             Expr::UnaryOp(op, expr) => {
//                 let val_reg = self.generate_expr(expr, local_vars)?;

//                 match op {
//                     UnaryOperator::Minus => {
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = sub i64 0, %{}", result_reg, val_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                     UnaryOperator::Not => {
//                         let cmp_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = icmp eq i64 %{}, 0", cmp_reg, val_reg).unwrap();
//                         let result_reg = self.next_temp();
//                         writeln!(self.output, "  %{} = zext i1 %{} to i64", result_reg, cmp_reg).unwrap();
//                         Ok(result_reg)
//                     }
//                 }
//             }

//             Expr::Cast(expr, target_type) => {
//                 let val_reg = self.generate_expr(expr, local_vars)?;
//                 // For now, just return the value as-is since we're working with i64
//                 // In a full implementation, you'd handle proper type conversions
//                 Ok(val_reg)
//             }

//             Expr::Call(name, args) => {
//                 let func_info = self.functions.get(name)
//                     .ok_or_else(|| format!("Undefined function: {}", name))?
//                     .clone();

//                 if args.len() != func_info.params.len() {
//                     return Err(format!(
//                         "Function {} expects {} arguments, got {}",
//                         name,
//                         func_info.params.len(),
//                         args.len()
//                     ));
//                 }

//                 let mut arg_regs = Vec::new();
//                 for (arg, param) in args.iter().zip(func_info.params.iter()) {
//                     let arg_reg = self.generate_expr(arg, local_vars)?;
                    
//                     // Convert argument to the expected parameter type
//                     let param_llvm_type = self.param_type_to_llvm(&param.param_type);
//                     if param_llvm_type != "i64" {
//                         let converted_reg = self.next_temp();
//                         self.cast_i64_to_type(arg_reg, &self.param_type_to_type(&param.param_type), converted_reg)?;
//                         arg_regs.push((converted_reg, param_llvm_type));
//                     } else {
//                         arg_regs.push((arg_reg, "i64".to_string()));
//                     }
//                 }

//                 let result_reg = self.next_temp();
//                 let arg_list = arg_regs.iter()
//                     .map(|(reg, ty)| format!("{} %{}", ty, reg))
//                     .collect::<Vec<_>>()
//                     .join(", ");

//                 writeln!(
//                     self.output,
//                     "  %{} = call i64 @{}({})",
//                     result_reg,
//                     name,
//                     arg_list
//                 ).unwrap();

//                 Ok(result_reg)
//             }
//         }
//     }

//     fn type_to_llvm(&self, ty: &Type) -> String {
//         match ty {
//             Type::I8 => "i8".to_string(),
//             Type::I16 => "i16".to_string(),
//             Type::I32 => "i32".to_string(),
//             Type::I64 => "i64".to_string(),
//             Type::I128 => "i128".to_string(),
//             Type::U8 => "i8".to_string(),
//             Type::U16 => "i16".to_string(),
//             Type::U32 => "i32".to_string(),
//             Type::U64 => "i64".to_string(),
//             Type::U128 => "i128".to_string(),
//             Type::F32 => "float".to_string(),
//             Type::F64 => "double".to_string(),
//             Type::Bool => "i1".to_string(),
//             Type::String => "ptr".to_string(),
//             Type::Any => "i64".to_string(), // Default to i64 for Any type
//         }
//     }

//     fn param_type_to_llvm(&self, param_type: &ParamType) -> String {
//         match param_type {
//             ParamType::Number(ty) => self.type_to_llvm(ty),
//             ParamType::String => "ptr".to_string(),
//             ParamType::Bool => "i1".to_string(),
//             ParamType::Any => "i64".to_string(),
//         }
//     }

//     fn param_type_to_type(&self, param_type: &ParamType) -> Type {
//         match param_type {
//             ParamType::Number(ty) => ty.clone(),
//             ParamType::String => Type::String,
//             ParamType::Bool => Type::Bool,
//             ParamType::Any => Type::Any,
//         }
//     }

//     fn convert_to_i64(&mut self, from_reg: u32, from_type: &ParamType, to_reg: u32) -> Result<(), String> {
//         match from_type {
//             ParamType::Number(Type::I8) => {
//                 writeln!(self.output, "  %{} = sext i8 %{} to i64", to_reg, from_reg).unwrap();
//             }
//             ParamType::Number(Type::I16) => {
//                 writeln!(self.output, "  %{} = sext i16 %{} to i64", to_reg, from_reg).unwrap();
//             }
//             ParamType::Number(Type::I32) => {
//                 writeln!(self.output, "  %{} = sext i32 %{} to i64", to_reg, from_reg).unwrap();
//             }
//             ParamType::Number(Type::I64) => {
//                 writeln!(self.output, "  %{} = add i64 0, %{}", to_reg, from_reg).unwrap();
//             }
//             ParamType::Bool => {
//                 writeln!(self.output, "  %{} = zext i1 %{} to i64", to_reg, from_reg).unwrap();
//             }
//             _ => {
//                 writeln!(self.output, "  %{} = add i64 0, %{}", to_reg, from_reg).unwrap();
//             }
//         }
//         Ok(())
//     }

//     fn cast_i64_to_type(&mut self, from_reg: u32, to_type: &Type, to_reg: u32) -> Result<(), String> {
//         match to_type {
//             Type::I8 => {
//                 writeln!(self.output, "  %{} = trunc i64 %{} to i8", to_reg, from_reg).unwrap();
//             }
//             Type::I16 => {
//                 writeln!(self.output, "  %{} = trunc i64 %{} to i16", to_reg, from_reg).unwrap();
//             }
//             Type::I32 => {
//                 writeln!(self.output, "  %{} = trunc i64 %{} to i32", to_reg, from_reg).unwrap();
//             }
//             Type::I64 => {
//                 writeln!(self.output, "  %{} = add i64 0, %{}", to_reg, from_reg).unwrap();
//             }
//             Type::Bool => {
//                 writeln!(self.output, "  %{} = icmp ne i64 %{}, 0", to_reg, from_reg).unwrap();
//             }
//             Type::F32 => {
//                 writeln!(self.output, "  %{} = sitofp i64 %{} to float", to_reg, from_reg).unwrap();
//             }
//             Type::F64 => {
//                 writeln!(self.output, "  %{} = sitofp i64 %{} to double", to_reg, from_reg).unwrap();
//             }
//             _ => {
//                 writeln!(self.output, "  %{} = add i64 0, %{}", to_reg, from_reg).unwrap();
//             }
//         }
//         Ok(())
//     }

//     fn next_temp(&mut self) -> u32 {
//         let temp = self.next_temp;
//         self.next_temp += 1;
//         temp
//     }

//     fn next_label(&mut self) -> String {
//         let label = format!("label{}", self.next_label);
//         self.next_label += 1;
//         label
//     }

//     fn has_return(&self, stmts: &[Stmt]) -> bool {
//         stmts.iter().any(|stmt| match stmt {
//             Stmt::Return(_) => true,
//             Stmt::If(_, then_body, else_body) => {
//                 self.has_return(then_body) || 
//                 else_body.as_ref().map_or(false, |body| self.has_return(body))
//             }
//             Stmt::While(_, body) => self.has_return(body),
//             Stmt::Block(body) => self.has_return(body),
//             _ => false,
//         })
//     }
// }

// // Main interface function that matches your usage pattern
// pub fn generate_llvm(main_func: &Function) -> String {
//     let mut codegen = LLVMCodegen::new();
    
//     // Generate LLVM header
//     codegen.generate_header();
    
//     // Add main function info to functions map
//     codegen.functions.insert(
//         main_func.name.clone(),
//         FunctionInfo {
//             params: main_func.params.clone(),
//             return_type: "i64".to_string(),
//         }
//     );
    
//     // Generate the main function
//     if let Err(e) = codegen.generate_function(main_func) {
//         eprintln!("Error generating LLVM IR: {}", e);
//         return String::new();
//     }
    
//     codegen.output
// }

// // Alternative function for full AST compilation
// pub fn compile_to_llvm(ast: &[Stmt]) -> Result<String, String> {
//     let mut codegen = LLVMCodegen::new();
//     codegen.generate(ast)
// }