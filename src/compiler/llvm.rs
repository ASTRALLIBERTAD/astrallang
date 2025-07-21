use crate::compiler::ast::*;
use std::collections::HashMap;

pub fn generate_llvm(func: &Function) -> String {
    let mut generator = LLVMGenerator::new();
    generator.generate_function(func)
}

#[derive(Debug, Clone)]
enum Value {
    Constant(i64),
    Register(String),
}

impl Value {
    fn to_llvm(&self) -> String {
        match self {
            Value::Constant(n) => n.to_string(),
            Value::Register(r) => r.clone(),
        }
    }
}

struct LLVMGenerator {
    label_counter: usize,
    variables: HashMap<String, String>, // variable name -> LLVM register
    register_counter: usize,
    code_buffer: Vec<String>,
    // Optimization state
    constant_values: HashMap<String, i64>, // track constant values in registers
    unused_registers: Vec<String>, // pool of unused registers for reuse
}

impl LLVMGenerator {
    fn new() -> Self {
        LLVMGenerator {
            label_counter: 0,
            variables: HashMap::new(),
            register_counter: 0,
            code_buffer: Vec::new(),
            constant_values: HashMap::new(),
            unused_registers: Vec::new(),
        }
    }

    fn next_register(&mut self) -> String {
        // Try to reuse an unused register first
        if let Some(reg) = self.unused_registers.pop() {
            reg
        } else {
            let reg = format!("%{}", self.register_counter);
            self.register_counter += 1;
            reg
        }
    }

    fn mark_register_unused(&mut self, reg: &str) {
        if reg.starts_with('%') && !self.unused_registers.contains(&reg.to_string()) {
            self.unused_registers.push(reg.to_string());
        }
    }

    fn next_label(&mut self) -> String {
        let label = format!("label{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn emit(&mut self, instruction: String) {
        self.code_buffer.push(instruction);
    }

    fn flush_code(&mut self) -> String {
        let result = self.code_buffer.join("");
        self.code_buffer.clear();
        result
    }

    // Constant folding for binary operations
    fn fold_binary_op(&self, lhs: i64, op: &BinaryOperator, rhs: i64) -> Option<i64> {
        match op {
            BinaryOperator::Add => Some(lhs.wrapping_add(rhs)),
            BinaryOperator::Subtract => Some(lhs.wrapping_sub(rhs)),
            BinaryOperator::Multiply => Some(lhs.wrapping_mul(rhs)),
            BinaryOperator::Divide => {
                if rhs != 0 { Some(lhs / rhs) } else { None }
            },
            BinaryOperator::Modulo => {
                if rhs != 0 { Some(lhs % rhs) } else { None }
            },
            BinaryOperator::Equal => Some(if lhs == rhs { 1 } else { 0 }),
            BinaryOperator::NotEqual => Some(if lhs != rhs { 1 } else { 0 }),
            BinaryOperator::Less => Some(if lhs < rhs { 1 } else { 0 }),
            BinaryOperator::Greater => Some(if lhs > rhs { 1 } else { 0 }),
            BinaryOperator::LessEqual => Some(if lhs <= rhs { 1 } else { 0 }),
            BinaryOperator::GreaterEqual => Some(if lhs >= rhs { 1 } else { 0 }),
        }
    }

    // Constant folding for unary operations
    fn fold_unary_op(&self, op: &UnaryOperator, operand: i64) -> i64 {
        match op {
            UnaryOperator::Minus => -operand,
            UnaryOperator::Not => if operand == 0 { 1 } else { 0 },
        }
    }

    fn generate_function(&mut self, func: &Function) -> String {
        let mut ir = String::new();
        
        // LLVM IR header
        ir.push_str("; Optimized LLVM IR for Astral language\n");
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
                let value = self.generate_expr_optimized(expr);
                
                match value {
                    Value::Constant(n) => {
                        // For constants, we can often avoid allocation and just track the value
                        let alloca_reg = self.next_register();
                        self.emit(format!("    {} = alloca i64\n", alloca_reg));
                        self.emit(format!("    store i64 {}, i64* {}\n", n, alloca_reg));
                        self.variables.insert(name.clone(), alloca_reg.clone());
                        // Track that this variable has a constant value
                        if let Some(var_reg) = self.variables.get(name) {
                            self.constant_values.insert(var_reg.clone(), n);
                        }
                    }
                    Value::Register(reg) => {
                        let alloca_reg = self.next_register();
                        self.emit(format!("    {} = alloca i64\n", alloca_reg));
                        self.emit(format!("    store i64 {}, i64* {}\n", reg, alloca_reg));
                        self.variables.insert(name.clone(), alloca_reg);
                        self.mark_register_unused(&reg);
                    }
                }
                
                self.flush_code()
            }
            
            Stmt::Print(expr) => {
                let value = self.generate_expr_optimized(expr);
                
                let fmt_reg = self.next_register();
                self.emit(format!("    {} = getelementptr [5 x i8], [5 x i8]* @fmt, i32 0, i32 0\n", fmt_reg));
                self.emit(format!("    call i32 (i8*, ...) @printf(i8* {}, i64 {})\n", fmt_reg, value.to_llvm()));
                
                if let Value::Register(reg) = value {
                    self.mark_register_unused(&reg);
                }
                
                self.flush_code()
            }
            
            Stmt::If(condition, then_body, else_body) => {
                let cond_value = self.generate_expr_optimized(condition);
                
                // Constant condition optimization
                if let Value::Constant(n) = cond_value {
                    if n != 0 {
                        // Condition is always true - only generate then branch
                        let mut code = String::new();
                        for stmt in then_body {
                            code.push_str(&self.generate_stmt(stmt));
                        }
                        return code;
                    } else {
                        // Condition is always false - only generate else branch
                        if let Some(else_stmts) = else_body {
                            let mut code = String::new();
                            for stmt in else_stmts {
                                code.push_str(&self.generate_stmt(stmt));
                            }
                            return code;
                        } else {
                            return String::new(); // No code needed
                        }
                    }
                }
                
                let then_label = self.next_label();
                let else_label = self.next_label();
                let end_label = self.next_label();
                
                let cmp_reg = self.next_register();
                self.emit(format!(
                    "    {} = icmp ne i64 {}, 0\n",
                    cmp_reg,
                    cond_value.to_llvm()
                ));
                self.emit(format!(
                    "    br i1 {}, label %{}, label %{}\n",
                    cmp_reg, then_label, else_label
                ));

                
                self.emit(format!("{}:\n", then_label));
                for stmt in then_body {
                    let stmt_code = self.generate_stmt(stmt);
                    self.emit(stmt_code);
                }
                self.emit(format!("    br label %{}\n", end_label));
                
                self.emit(format!("{}:\n", else_label));
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        let stmt_code = self.generate_stmt(stmt);
                        self.emit(stmt_code);
                    }
                }
                self.emit(format!("    br label %{}\n", end_label));
                
                self.emit(format!("{}:\n", end_label));
                
                if let Value::Register(reg) = cond_value {
                    self.mark_register_unused(&reg);
                }
                
                self.flush_code()
            }
            
            Stmt::While(condition, body) => {
                // Check if it's an infinite loop or never-executed loop
                if let Value::Constant(n) = self.generate_expr_optimized(condition) {
                    if n == 0 {
                        // Loop never executes
                        return String::new();
                    }
                    // Note: infinite loops (n != 0) still need proper code generation
                    // for break statements and other control flow
                }
                
                let loop_label = self.next_label();
                let body_label = self.next_label();
                let end_label = self.next_label();
                
                self.emit(format!("    br label %{}\n", loop_label));
                self.emit(format!("{}:\n", loop_label));
                
                let cond_value = self.generate_expr_optimized(condition);
                
                let cmp_reg = self.next_register();
                self.emit(format!("    {} = icmp ne i64 {}, 0\n", cmp_reg, cond_value.to_llvm()));
                self.emit(format!("    br i1 {}, label %{}, label %{}\n", cmp_reg, body_label, end_label));
                
                self.emit(format!("{}:\n", body_label));
                for stmt in body {
                    let stmt_code = self.generate_stmt(stmt);
                    self.emit(stmt_code);
                }
                self.emit(format!("    br label %{}\n", loop_label));
                
                self.emit(format!("{}:\n", end_label));
                
                if let Value::Register(reg) = cond_value {
                    self.mark_register_unused(&reg);
                }
                
                self.flush_code()
            }
            
            Stmt::Expression(expr) => {
                let value = self.generate_expr_optimized(expr);
                if let Value::Register(reg) = value {
                    self.mark_register_unused(&reg);
                }
                self.flush_code()
            }
            
            _ => String::new(),
        }
    }

    fn generate_expr_optimized(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Constant(*n),
            
            Expr::Ident(name) => {
                if let Some(var_reg) = self.variables.get(name).cloned() {
                    // Check if we know this variable's constant value
                    if let Some(&const_val) = self.constant_values.get(&var_reg) {
                        Value::Constant(const_val)
                    } else {
                        let load_reg = self.next_register();
                        self.emit(format!("    {} = load i64, i64* {}\n", load_reg, var_reg));
                        Value::Register(load_reg)
                    }
                } else {
                    let reg = self.next_register();
                    self.emit(format!("    {} = add i64 0, 0  ; undefined variable {}\n", reg, name));
                    Value::Register(reg)
                }
            }
            
            Expr::BinaryOp(lhs, op, rhs) => {
                let lhs_val = self.generate_expr_optimized(lhs);
                let rhs_val = self.generate_expr_optimized(rhs);
                
                // Constant folding
                if let (Value::Constant(l), Value::Constant(r)) = (&lhs_val, &rhs_val) {
                    if let Some(result) = self.fold_binary_op(*l, op, *r) {
                        return Value::Constant(result);
                    }
                }
                
                // Strength reduction optimizations
                match (op, &lhs_val, &rhs_val) {
                    // x + 0 = x, 0 + x = x
                    (BinaryOperator::Add, Value::Constant(0), val) | 
                    (BinaryOperator::Add, val, Value::Constant(0)) => val.clone(),
                    
                    // x - 0 = x
                    (BinaryOperator::Subtract, val, Value::Constant(0)) => val.clone(),
                    
                    // x * 0 = 0, 0 * x = 0
                    (BinaryOperator::Multiply, Value::Constant(0), _) | 
                    (BinaryOperator::Multiply, _, Value::Constant(0)) => Value::Constant(0),
                    
                    // x * 1 = x, 1 * x = x
                    (BinaryOperator::Multiply, Value::Constant(1), val) | 
                    (BinaryOperator::Multiply, val, Value::Constant(1)) => val.clone(),
                    
                    // x * 2 -> x << 1 (left shift), but LLVM will optimize this anyway
                    // x / 1 = x
                    (BinaryOperator::Divide, val, Value::Constant(1)) => val.clone(),
                    
                    _ => {
                        // Generate normal binary operation
                        let result_reg = self.next_register();
                        
                        let llvm_op = match op {
                            BinaryOperator::Add => "add",
                            BinaryOperator::Subtract => "sub", 
                            BinaryOperator::Multiply => "mul",
                            BinaryOperator::Divide => "sdiv",
                            BinaryOperator::Modulo => "srem",
                            _ => {
                                // Comparison operations
                                let cmp_op = match op {
                                    BinaryOperator::Equal => "eq",
                                    BinaryOperator::NotEqual => "ne", 
                                    BinaryOperator::Less => "slt",
                                    BinaryOperator::Greater => "sgt",
                                    BinaryOperator::LessEqual => "sle",
                                    BinaryOperator::GreaterEqual => "sge",
                                    _ => unreachable!(),
                                };
                                
                                let cmp_reg = self.next_register();
                                self.emit(format!("    {} = icmp {} i64 {}, {}\n", cmp_reg, cmp_op, lhs_val.to_llvm(), rhs_val.to_llvm()));
                                self.emit(format!("    {} = zext i1 {} to i64\n", result_reg, cmp_reg));
                                
                                // Mark input registers as unused
                                if let Value::Register(reg) = lhs_val { self.mark_register_unused(&reg); }
                                if let Value::Register(reg) = rhs_val { self.mark_register_unused(&reg); }
                                
                                return Value::Register(result_reg);
                            }
                        };
                        
                        self.emit(format!("    {} = {} i64 {}, {}\n", result_reg, llvm_op, lhs_val.to_llvm(), rhs_val.to_llvm()));
                        
                        // Mark input registers as unused
                        if let Value::Register(reg) = lhs_val { self.mark_register_unused(&reg); }
                        if let Value::Register(reg) = rhs_val { self.mark_register_unused(&reg); }
                        
                        Value::Register(result_reg)
                    }
                }
            }
            
            Expr::UnaryOp(op, operand) => {
                let operand_val = self.generate_expr_optimized(operand);
                
                // Constant folding
                if let Value::Constant(n) = operand_val {
                    return Value::Constant(self.fold_unary_op(op, n));
                }
                
                // Optimizations for specific cases
                match op {
                    UnaryOperator::Minus => {
                        let result_reg = self.next_register();
                        self.emit(format!("    {} = sub i64 0, {}\n", result_reg, operand_val.to_llvm()));
                        if let Value::Register(reg) = operand_val { self.mark_register_unused(&reg); }
                        Value::Register(result_reg)
                    }
                    UnaryOperator::Not => {
                        let cmp_reg = self.next_register();
                        let result_reg = self.next_register();
                        self.emit(format!("    {} = icmp eq i64 {}, 0\n", cmp_reg, operand_val.to_llvm()));
                        self.emit(format!("    {} = zext i1 {} to i64\n", result_reg, cmp_reg));
                        if let Value::Register(reg) = operand_val { self.mark_register_unused(&reg); }
                        Value::Register(result_reg)
                    }
                }
            }
            
            _ => {
                let reg = self.next_register();
                self.emit(format!("    {} = add i64 0, 0  ; unimplemented expression\n", reg));
                Value::Register(reg)
            }
        }
    }

    fn branch(&mut self, cond_reg: &str, then_label: &str, else_label: &str) -> String {
        let cmp_reg = self.next_register();
        let mut code = String::new();
        code.push_str(&format!("    {} = icmp ne i64 {}, 0\n", cmp_reg, cond_reg));
        code.push_str(&format!("    br i1 {}, label %{}, label %{}\n", cmp_reg, then_label, else_label));
        code
    }

    fn store_string_literal(&mut self, s: &str) -> String {
        let name = format!("@.str{}", self.label_counter);
        self.label_counter += 1;
        let len = s.len() + 1;
        let escaped = s.escape_default().to_string();
        self.code_buffer.insert(
            0,
            format!(
                "{} = private constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ),
        );
        name
    }

}