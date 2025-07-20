use crate::compiler::ast::*;
use std::collections::HashMap;

pub fn generate_llvm(func: &Function) -> String {
    let mut generator = LLVMGenerator::new();
    generator.generate_function(func)
}

struct LLVMGenerator {
    label_counter: usize,
    variables: HashMap<String, String>, // variable name -> LLVM register
    register_counter: usize,
}

impl LLVMGenerator {
    fn new() -> Self {
        LLVMGenerator {
            label_counter: 0,
            variables: HashMap::new(),
            register_counter: 1,
        }
    }

    fn next_register(&mut self) -> String {
        let reg = format!("%{}", self.register_counter);
        self.register_counter += 1;
        reg
    }

    fn next_label(&mut self) -> String {
        let label = format!("label{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn generate_function(&mut self, func: &Function) -> String {
        let mut ir = String::new();
        
        // LLVM IR header
        ir.push_str("; Generated LLVM IR for Astral language\n");
        ir.push_str("target triple = \"aarch64-linux-android\"\n\n");
        
        // Declare external functions
        ir.push_str("declare i32 @printf(i8*, ...)\n");
        ir.push_str("declare void @exit(i32)\n\n");
        
        // Format string for printing
        ir.push_str("@fmt = private constant [5 x i8] c\"%ld\\0A\\00\"\n\n");
        
        // Main function
        ir.push_str("define i32 @main() {\n");
        ir.push_str("entry:\n");
        
        // Generate function body
        for stmt in &func.body {
            ir.push_str(&self.generate_stmt(stmt));
        }
        
        // Return 0 and close function
        ir.push_str("    ret i32 0\n");
        ir.push_str("}\n");
        
        ir
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Let(name, expr) => {
                let (expr_code, result_reg) = self.generate_expr_with_result(expr);
                let alloca_reg = self.next_register();
                
                let mut code = format!("    {} = alloca i64\n", alloca_reg);
                code.push_str(&expr_code);
                code.push_str(&format!("    store i64 {}, i64* {}\n", result_reg, alloca_reg));
                
                self.variables.insert(name.clone(), alloca_reg);
                code
            }
            Stmt::Print(expr) => {
                let (expr_code, result_reg) = self.generate_expr_with_result(expr);
                let mut code = expr_code;
                
                let fmt_reg = self.next_register();
                code.push_str(&format!("    {} = getelementptr [5 x i8], [5 x i8]* @fmt, i32 0, i32 0\n", fmt_reg));
                code.push_str(&format!("    call i32 (i8*, ...) @printf(i8* {}, i64 {})\n", fmt_reg, result_reg));
                
                code
            }
            Stmt::If(condition, then_body, else_body) => {
                let (cond_code, cond_reg) = self.generate_expr_with_result(condition);
                let then_label = self.next_label();
                let else_label = self.next_label();
                let end_label = self.next_label();
                
                let mut code = cond_code;
                let cmp_reg = self.next_register();
                code.push_str(&format!("    {} = icmp ne i64 {}, 0\n", cmp_reg, cond_reg));
                code.push_str(&format!("    br i1 {}, label %{}, label %{}\n", cmp_reg, then_label, else_label));
                
                code.push_str(&format!("{}:\n", then_label));
                for stmt in then_body {
                    code.push_str(&self.generate_stmt(stmt));
                }
                code.push_str(&format!("    br label %{}\n", end_label));
                
                code.push_str(&format!("{}:\n", else_label));
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        code.push_str(&self.generate_stmt(stmt));
                    }
                }
                code.push_str(&format!("    br label %{}\n", end_label));
                
                code.push_str(&format!("{}:\n", end_label));
                code
            }
            Stmt::While(condition, body) => {
                let loop_label = self.next_label();
                let body_label = self.next_label();
                let end_label = self.next_label();
                
                let mut code = format!("    br label %{}\n", loop_label);
                code.push_str(&format!("{}:\n", loop_label));
                
                let (cond_code, cond_reg) = self.generate_expr_with_result(condition);
                code.push_str(&cond_code);
                
                let cmp_reg = self.next_register();
                code.push_str(&format!("    {} = icmp ne i64 {}, 0\n", cmp_reg, cond_reg));
                code.push_str(&format!("    br i1 {}, label %{}, label %{}\n", cmp_reg, body_label, end_label));
                
                code.push_str(&format!("{}:\n", body_label));
                for stmt in body {
                    code.push_str(&self.generate_stmt(stmt));
                }
                code.push_str(&format!("    br label %{}\n", loop_label));
                
                code.push_str(&format!("{}:\n", end_label));
                code
            }
            Stmt::Expression(expr) => {
                let (expr_code, _) = self.generate_expr_with_result(expr);
                expr_code
            }
            _ => String::new(),
        }
    }

    fn generate_expr_with_result(&mut self, expr: &Expr) -> (String, String) {
        match expr {
            Expr::Number(n) => {
                let reg = self.next_register();
                (format!("    {} = add i64 0, {}\n", reg, n), reg)
            }
            Expr::Ident(name) => {
                if let Some(var_reg) = self.variables.get(name).cloned() {
                    let load_reg = self.next_register();
                    (format!("    {} = load i64, i64* {}\n", load_reg, var_reg), load_reg)
                } else {
                    let reg = self.next_register();
                    (format!("    {} = add i64 0, 0  ; undefined variable {}\n", reg, name), reg)
                }
            }
            Expr::BinaryOp(lhs, op, rhs) => {
                let (lhs_code, lhs_reg) = self.generate_expr_with_result(lhs);
                let (rhs_code, rhs_reg) = self.generate_expr_with_result(rhs);
                let result_reg = self.next_register();
                
                let mut code = lhs_code;
                code.push_str(&rhs_code);
                
                let op_instr = match op {
                    BinaryOperator::Add => format!("add i64 {}, {}", lhs_reg, rhs_reg),
                    BinaryOperator::Subtract => format!("sub i64 {}, {}", lhs_reg, rhs_reg),
                    BinaryOperator::Multiply => format!("mul i64 {}, {}", lhs_reg, rhs_reg),
                    BinaryOperator::Divide => format!("sdiv i64 {}, {}", lhs_reg, rhs_reg),
                    BinaryOperator::Modulo => format!("srem i64 {}, {}", lhs_reg, rhs_reg),
                    BinaryOperator::Equal => {
                        let cmp_reg = self.next_register();
                        code.push_str(&format!("    {} = icmp eq i64 {}, {}\n", cmp_reg, lhs_reg, rhs_reg));
                        format!("zext i1 {} to i64", cmp_reg)
                    }
                    BinaryOperator::NotEqual => {
                        let cmp_reg = self.next_register();
                        code.push_str(&format!("    {} = icmp ne i64 {}, {}\n", cmp_reg, lhs_reg, rhs_reg));
                        format!("zext i1 {} to i64", cmp_reg)
                    }
                    BinaryOperator::Less => {
                        let cmp_reg = self.next_register();
                        code.push_str(&format!("    {} = icmp slt i64 {}, {}\n", cmp_reg, lhs_reg, rhs_reg));
                        format!("zext i1 {} to i64", cmp_reg)
                    }
                    BinaryOperator::Greater => {
                        let cmp_reg = self.next_register();
                        code.push_str(&format!("    {} = icmp sgt i64 {}, {}\n", cmp_reg, lhs_reg, rhs_reg));
                        format!("zext i1 {} to i64", cmp_reg)
                    }
                    BinaryOperator::LessEqual => {
                        let cmp_reg = self.next_register();
                        code.push_str(&format!("    {} = icmp sle i64 {}, {}\n", cmp_reg, lhs_reg, rhs_reg));
                        format!("zext i1 {} to i64", cmp_reg)
                    }
                    BinaryOperator::GreaterEqual => {
                        let cmp_reg = self.next_register();
                        code.push_str(&format!("    {} = icmp sge i64 {}, {}\n", cmp_reg, lhs_reg, rhs_reg));
                        format!("zext i1 {} to i64", cmp_reg)
                    }
                };
                
                code.push_str(&format!("    {} = {}\n", result_reg, op_instr));
                (code, result_reg)
            }
            Expr::UnaryOp(op, expr) => {
                let (expr_code, expr_reg) = self.generate_expr_with_result(expr);
                let result_reg = self.next_register();
                
                let mut code = expr_code;
                match op {
                    UnaryOperator::Minus => {
                        code.push_str(&format!("    {} = sub i64 0, {}\n", result_reg, expr_reg));
                    }
                    UnaryOperator::Not => {
                        let cmp_reg = self.next_register();
                        code.push_str(&format!("    {} = icmp eq i64 {}, 0\n", cmp_reg, expr_reg));
                        code.push_str(&format!("    {} = zext i1 {} to i64\n", result_reg, cmp_reg));
                    }
                }
                
                (code, result_reg)
            }
            _ => {
                let reg = self.next_register();
                (format!("    {} = add i64 0, 0  ; unimplemented expression\n", reg), reg)
            }
        }
    }
}