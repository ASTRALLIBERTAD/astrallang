use crate::compiler::ast::*;
use std::collections::HashMap;
use std::fmt::Write;

pub struct LLVMCodegen {
    output: String,
    next_label: u32,
    next_temp: u32,
    functions: HashMap<String, FunctionInfo>,
}

#[derive(Clone)]
struct FunctionInfo {
    params: Vec<String>,
    return_type: String,
}

impl LLVMCodegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            next_label: 0,
            next_temp: 0,
            functions: HashMap::new(),
        }
    }

    pub fn generate(&mut self, stmts: &[Stmt]) -> Result<String, String> {
        // Generate LLVM header and declarations
        self.generate_header();

        // First pass: collect function signatures
        for stmt in stmts {
            if let Stmt::Function(func) = stmt {
                self.functions.insert(
                    func.name.clone(),
                    FunctionInfo {
                        params: func.params.iter().map(|p| p.name.clone()).collect(),
                        return_type: "i64".to_string(), // Assuming all functions return i64 for simplicity
                    }
                );
            }
        }

        // Second pass: generate function definitions
        for stmt in stmts {
            if let Stmt::Function(func) = stmt {
                self.generate_function(func)?;
            }
        }

        Ok(self.output.clone())
    }

    fn generate_header(&mut self) {
        writeln!(self.output, "; LLVM IR generated from custom language").unwrap();
        writeln!(self.output, "target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"").unwrap();
        writeln!(self.output, "target triple = \"x86_64-unknown-linux-gnu\"").unwrap();
        writeln!(self.output).unwrap();

        // Declare external functions
        writeln!(self.output, "declare i32 @printf(i8*, ...)").unwrap();
        writeln!(self.output, "declare i8* @malloc(i64)").unwrap();
        writeln!(self.output, "declare void @free(i8*)").unwrap();
        writeln!(self.output).unwrap();

        // Global string constants for printing
        writeln!(self.output, "@.str.num = private unnamed_addr constant [5 x i8] c\"%ld\\0A\\00\", align 1").unwrap();
        writeln!(self.output, "@.str.bool_true = private unnamed_addr constant [6 x i8] c\"true\\0A\\00\", align 1").unwrap();
        writeln!(self.output, "@.str.bool_false = private unnamed_addr constant [7 x i8] c\"false\\0A\\00\", align 1").unwrap();
        writeln!(self.output, "@.str.null = private unnamed_addr constant [6 x i8] c\"null\\0A\\00\", align 1").unwrap();
        writeln!(self.output, "@.str.str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\", align 1").unwrap();
        writeln!(self.output).unwrap();
    }

    fn generate_function(&mut self, func: &Function) -> Result<(), String> {
        // Reset temp counter for each function
        self.next_temp = func.params.len() as u32;

        // Function signature with parameter names
        let mut param_decl = String::new();
        for (i, _param) in func.params.iter().enumerate() {
            if i > 0 {
                param_decl.push_str(", ");
            }
            param_decl.push_str(&format!("i64 %{}", i));
        }

        writeln!(
            self.output,
            "define i64 @{}({}) {{",
            func.name,
            param_decl
        ).unwrap();

        // Entry block
        writeln!(self.output, "entry:").unwrap();

        // Allocate space for parameters
        let mut local_vars = HashMap::new();
        for (i, param) in func.params.iter().enumerate() {
            let alloca = self.next_temp();
            writeln!(self.output, "  %{} = alloca i64, align 8", alloca).unwrap();
            writeln!(self.output, "  store i64 %{}, i64* %{}, align 8", i, alloca).unwrap();
            local_vars.insert(param.name.clone(), alloca);
        }

        // Generate function body
        let result = self.generate_block(&func.body, &mut local_vars)?;

        // If no explicit return, return 0
        if !self.has_return(&func.body) {
            writeln!(self.output, "  ret i64 0").unwrap();
        }

        writeln!(self.output, "}}").unwrap();
        writeln!(self.output).unwrap();

        Ok(())
    }

    fn generate_block(&mut self, stmts: &[Stmt], local_vars: &mut HashMap<String, u32>) -> Result<Option<u32>, String> {
        for stmt in stmts {
            match stmt {
                Stmt::Let(name, expr) => {
                    let val_reg = self.generate_expr(expr, local_vars)?;
                    let alloca = self.next_temp();
                    writeln!(self.output, "  %{} = alloca i64, align 8", alloca).unwrap();
                    writeln!(self.output, "  store i64 %{}, i64* %{}, align 8", val_reg, alloca).unwrap();
                    local_vars.insert(name.clone(), alloca);
                }

                Stmt::Print(expr) => {
                    let val_reg = self.generate_expr(expr, local_vars)?;
                    // For simplicity, assuming all values are numbers
                    let temp = self.next_temp();
                    writeln!(
                        self.output,
                        "  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.num, i64 0, i64 0), i64 %{})",
                        temp,
                        val_reg
                    ).unwrap();
                }

                Stmt::Return(Some(expr)) => {
                    let val_reg = self.generate_expr(expr, local_vars)?;
                    writeln!(self.output, "  ret i64 %{}", val_reg).unwrap();
                    return Ok(Some(val_reg));
                }

                Stmt::Return(None) => {
                    writeln!(self.output, "  ret i64 0").unwrap();
                    return Ok(Some(0)); // Dummy register
                }

                Stmt::If(condition, then_body, else_body) => {
                    let cond_reg = self.generate_expr(condition, local_vars)?;
                    let then_label = self.next_label();
                    let else_label = self.next_label();
                    let end_label = self.next_label();

                    // Convert condition to boolean
                    let bool_reg = self.next_temp();
                    writeln!(self.output, "  %{} = icmp ne i64 %{}, 0", bool_reg, cond_reg).unwrap();
                    writeln!(self.output, "  br i1 %{}, label %{}, label %{}", bool_reg, then_label, else_label).unwrap();

                    // Then block
                    writeln!(self.output, "{}:", then_label).unwrap();
                    self.generate_block(then_body, local_vars)?;
                    writeln!(self.output, "  br label %{}", end_label).unwrap();

                    // Else block
                    writeln!(self.output, "{}:", else_label).unwrap();
                    if let Some(else_stmts) = else_body {
                        self.generate_block(else_stmts, local_vars)?;
                    }
                    writeln!(self.output, "  br label %{}", end_label).unwrap();

                    // End block
                    writeln!(self.output, "{}:", end_label).unwrap();
                }

                Stmt::While(condition, body) => {
                    let loop_label = self.next_label();
                    let body_label = self.next_label();
                    let end_label = self.next_label();

                    writeln!(self.output, "  br label %{}", loop_label).unwrap();

                    // Loop condition check
                    writeln!(self.output, "{}:", loop_label).unwrap();
                    let cond_reg = self.generate_expr(condition, local_vars)?;
                    let bool_reg = self.next_temp();
                    writeln!(self.output, "  %{} = icmp ne i64 %{}, 0", bool_reg, cond_reg).unwrap();
                    writeln!(self.output, "  br i1 %{}, label %{}, label %{}", bool_reg, body_label, end_label).unwrap();

                    // Loop body
                    writeln!(self.output, "{}:", body_label).unwrap();
                    self.generate_block(body, local_vars)?;
                    writeln!(self.output, "  br label %{}", loop_label).unwrap();

                    // End of loop
                    writeln!(self.output, "{}:", end_label).unwrap();
                }

                Stmt::Expression(expr) => {
                    self.generate_expr(expr, local_vars)?;
                }

                Stmt::Function(_) => {
                    // Function definitions are handled separately
                }
            }
        }
        Ok(None)
    }

    fn generate_expr(&mut self, expr: &Expr, local_vars: &HashMap<String, u32>) -> Result<u32, String> {
        match expr {
            Expr::Number(n) => {
                let reg = self.next_temp();
                writeln!(self.output, "  %{} = add i64 0, {}", reg, n).unwrap();
                Ok(reg)
            }

            Expr::Bool(b) => {
                let reg = self.next_temp();
                let val = if *b { 1 } else { 0 };
                writeln!(self.output, "  %{} = add i64 0, {}", reg, val).unwrap();
                Ok(reg)
            }

            Expr::Ident(name) => {
                let alloca = local_vars.get(name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))?;
                let reg = self.next_temp();
                writeln!(self.output, "  %{} = load i64, i64* %{}, align 8", reg, alloca).unwrap();
                Ok(reg)
            }

            Expr::String(_s) => {
                // For simplicity, strings are treated as their length
                // In a real implementation, you'd want proper string handling
                let reg = self.next_temp();
                writeln!(self.output, "  %{} = add i64 0, {}", reg, _s.len()).unwrap();
                Ok(reg)
            }

            Expr::BinaryOp(lhs, op, rhs) => {
                let left_reg = self.generate_expr(lhs, local_vars)?;
                let right_reg = self.generate_expr(rhs, local_vars)?;

                match op {
                    BinaryOperator::Add => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = add i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        Ok(result_reg)
                    }
                    BinaryOperator::Subtract => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = sub i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        Ok(result_reg)
                    }
                    BinaryOperator::Multiply => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = mul i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        Ok(result_reg)
                    }
                    BinaryOperator::Divide => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = sdiv i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        Ok(result_reg)
                    }
                    BinaryOperator::Modulo => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = srem i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        Ok(result_reg)
                    }
                    BinaryOperator::Equal => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = icmp eq i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        let zext_reg = self.next_temp();
                        writeln!(self.output, "  %{} = zext i1 %{} to i64", zext_reg, result_reg).unwrap();
                        Ok(zext_reg)
                    }
                    BinaryOperator::NotEqual => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = icmp ne i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        let zext_reg = self.next_temp();
                        writeln!(self.output, "  %{} = zext i1 %{} to i64", zext_reg, result_reg).unwrap();
                        Ok(zext_reg)
                    }
                    BinaryOperator::Less => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = icmp slt i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        let zext_reg = self.next_temp();
                        writeln!(self.output, "  %{} = zext i1 %{} to i64", zext_reg, result_reg).unwrap();
                        Ok(zext_reg)
                    }
                    BinaryOperator::Greater => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = icmp sgt i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        let zext_reg = self.next_temp();
                        writeln!(self.output, "  %{} = zext i1 %{} to i64", zext_reg, result_reg).unwrap();
                        Ok(zext_reg)
                    }
                    BinaryOperator::LessEqual => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = icmp sle i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        let zext_reg = self.next_temp();
                        writeln!(self.output, "  %{} = zext i1 %{} to i64", zext_reg, result_reg).unwrap();
                        Ok(zext_reg)
                    }
                    BinaryOperator::GreaterEqual => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = icmp sge i64 %{}, %{}", result_reg, left_reg, right_reg).unwrap();
                        let zext_reg = self.next_temp();
                        writeln!(self.output, "  %{} = zext i1 %{} to i64", zext_reg, result_reg).unwrap();
                        Ok(zext_reg)
                    }
                }
            }

            Expr::UnaryOp(op, expr) => {
                let val_reg = self.generate_expr(expr, local_vars)?;

                match op {
                    UnaryOperator::Minus => {
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = sub i64 0, %{}", result_reg, val_reg).unwrap();
                        Ok(result_reg)
                    }
                    UnaryOperator::Not => {
                        let cmp_reg = self.next_temp();
                        writeln!(self.output, "  %{} = icmp eq i64 %{}, 0", cmp_reg, val_reg).unwrap();
                        let result_reg = self.next_temp();
                        writeln!(self.output, "  %{} = zext i1 %{} to i64", result_reg, cmp_reg).unwrap();
                        Ok(result_reg)
                    }
                }
            }

            Expr::Call(name, args) => {
                let func_info = self.functions.get(name)
                    .ok_or_else(|| format!("Undefined function: {}", name))?;

                if args.len() != func_info.params.len() {
                    return Err(format!(
                        "Function {} expects {} arguments, got {}",
                        name,
                        func_info.params.len(),
                        args.len()
                    ));
                }

                let mut arg_regs = Vec::new();
                for arg in args {
                    let arg_reg = self.generate_expr(arg, local_vars)?;
                    arg_regs.push(arg_reg);
                }

                let result_reg = self.next_temp();
                let arg_list = arg_regs.iter()
                    .map(|reg| format!("i64 %{}", reg))
                    .collect::<Vec<_>>()
                    .join(", ");

                writeln!(
                    self.output,
                    "  %{} = call i64 @{}({})",
                    result_reg,
                    name,
                    arg_list
                ).unwrap();

                Ok(result_reg)
            }
        }
    }

    fn next_temp(&mut self) -> u32 {
        let temp = self.next_temp;
        self.next_temp += 1;
        temp
    }

    fn next_label(&mut self) -> String {
        let label = format!("label{}", self.next_label);
        self.next_label += 1;
        label
    }

    fn has_return(&self, stmts: &[Stmt]) -> bool {
        stmts.iter().any(|stmt| matches!(stmt, Stmt::Return(_)))
    }
}

// Main interface function that matches your usage pattern
pub fn generate_llvm(main_func: &Function) -> String {
    let mut codegen = LLVMCodegen::new();
    
    // Generate LLVM header
    codegen.generate_header();
    
    // Add main function info to functions map
    codegen.functions.insert(
        main_func.name.clone(),
        FunctionInfo {
            params: main_func.params.iter().map(|p| p.name.clone()).collect(),
            return_type: "i64".to_string(),
        }
    );
    
    // Reset temp counter before generating function
    codegen.next_temp = 0;
    
    // Generate the main function
    if let Err(e) = codegen.generate_function(main_func) {
        eprintln!("Error generating LLVM IR: {}", e);
        return String::new();
    }
    
    codegen.output
}

// Alternative function for full AST compilation
pub fn compile_to_llvm(ast: &[Stmt]) -> Result<String, String> {
    let mut codegen = LLVMCodegen::new();
    codegen.generate(ast)
}