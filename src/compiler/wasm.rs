use crate::compiler::ast::*;
use std::collections::HashMap;

pub fn generate_wasm(func: &Function) -> String {
    let mut generator = WASMGenerator::new();
    generator.generate_function(func)
}

struct WASMGenerator {
    variables: HashMap<String, u32>, // variable name -> local index
    local_count: u32,
    label_counter: usize,
}

impl WASMGenerator {
    fn new() -> Self {
        WASMGenerator {
            variables: HashMap::new(),
            local_count: 0,
            label_counter: 0,
        }
    }

    fn next_local(&mut self) -> u32 {
        let local = self.local_count;
        self.local_count += 1;
        local
    }

    fn next_label(&mut self) -> String {
        let label = format!("$label{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn generate_function(&mut self, func: &Function) -> String {
        let mut wasm = String::new();
        
        // WebAssembly Text Format (WAT)
        wasm.push_str("(module\n");
        wasm.push_str("  ;; Import print function from JavaScript\n");
        wasm.push_str("  (import \"env\" \"print\" (func $print (param i64)))\n\n");
        
        // Export main function
        wasm.push_str("  (func $main (export \"main\")\n");
        
        // Declare locals for variables
        if self.local_count > 0 {
            wasm.push_str(&format!("    (local i64)\n"));
        }
        
        // Generate function body
        for stmt in &func.body {
            wasm.push_str(&self.generate_stmt(stmt, 2));
        }
        
        wasm.push_str("  )\n");
        wasm.push_str(")\n");
        
        wasm
    }

    fn generate_stmt(&mut self, stmt: &Stmt, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        
        match stmt {
            Stmt::Let(name, expr) => {
                let local_idx = self.next_local();
                self.variables.insert(name.clone(), local_idx);
                
                let mut code = format!("{}; let {} = ...\n", indent_str, name);
                code.push_str(&self.generate_expr(expr, indent));
                code.push_str(&format!("{}local.set {}\n", indent_str, local_idx));
                code
            }
            Stmt::Print(expr) => {
                let mut code = format!("{}; print(...)\n", indent_str);
                code.push_str(&self.generate_expr(expr, indent));
                code.push_str(&format!("{}call $print\n", indent_str));
                code
            }
            Stmt::If(condition, then_body, else_body) => {
                let mut code = format!("{}; if statement\n", indent_str);
                code.push_str(&self.generate_expr(condition, indent));
                code.push_str(&format!("{}if (result i32)\n", indent_str));
                
                // Then block
                for stmt in then_body {
                    code.push_str(&self.generate_stmt(stmt, indent + 1));
                }
                
                // Else block
                if let Some(else_stmts) = else_body {
                    code.push_str(&format!("{}else\n", indent_str));
                    for stmt in else_stmts {
                        code.push_str(&self.generate_stmt(stmt, indent + 1));
                    }
                }
                
                code.push_str(&format!("{}end\n", indent_str));
                code
            }
            Stmt::While(condition, body) => {
                let loop_label = self.next_label();
                let mut code = format!("{}; while loop\n", indent_str);
                code.push_str(&format!("{}loop {}\n", indent_str, loop_label));
                
                // Check condition
                code.push_str(&self.generate_expr(condition, indent + 1));
                code.push_str(&format!("{}  i32.eqz\n", indent_str));
                code.push_str(&format!("{}  br_if 1  ; break if condition false\n", indent_str));
                
                // Loop body
                for stmt in body {
                    code.push_str(&self.generate_stmt(stmt, indent + 1));
                }
                
                code.push_str(&format!("{}  br {}  ; continue loop\n", indent_str, loop_label));
                code.push_str(&format!("{}end\n", indent_str));
                code
            }
            Stmt::Expression(expr) => {
                let mut code = format!("{}; expression statement\n", indent_str);
                code.push_str(&self.generate_expr(expr, indent));
                code.push_str(&format!("{}drop  ; discard result\n", indent_str));
                code
            }
            _ => String::new(),
        }
    }

    fn generate_expr(&mut self, expr: &Expr, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        
        match expr {
            Expr::Number(n) => {
                format!("{}i64.const {}\n", indent_str, n)
            }
            Expr::Ident(name) => {
                if let Some(&local_idx) = self.variables.get(name) {
                    format!("{}local.get {}\n", indent_str, local_idx)
                } else {
                    format!("{}i64.const 0  ;; undefined variable {}\n", indent_str, name)
                }
            }
            Expr::BinaryOp(lhs, op, rhs) => {
                let mut code = self.generate_expr(lhs, indent);
                code.push_str(&self.generate_expr(rhs, indent));
                
                let op_instr = match op {
                    BinaryOperator::Add => "i64.add",
                    BinaryOperator::Subtract => "i64.sub",
                    BinaryOperator::Multiply => "i64.mul",
                    BinaryOperator::Divide => "i64.div_s",
                    BinaryOperator::Modulo => "i64.rem_s",
                    BinaryOperator::Equal => "i64.eq",
                    BinaryOperator::NotEqual => "i64.ne",
                    BinaryOperator::Less => "i64.lt_s",
                    BinaryOperator::Greater => "i64.gt_s",
                    BinaryOperator::LessEqual => "i64.le_s",
                    BinaryOperator::GreaterEqual => "i64.ge_s",
                };
                
                code.push_str(&format!("{}{}\n", indent_str, op_instr));
                
                // Convert comparison results from i32 to i64
                if matches!(op, 
                    BinaryOperator::Equal | BinaryOperator::NotEqual | 
                    BinaryOperator::Less | BinaryOperator::Greater |
                    BinaryOperator::LessEqual | BinaryOperator::GreaterEqual
                ) {
                    code.push_str(&format!("{}i64.extend_i32_u\n", indent_str));
                }
                
                code
            }
            Expr::UnaryOp(op, expr) => {
                let mut code = self.generate_expr(expr, indent);
                
                match op {
                    UnaryOperator::Minus => {
                        code.push_str(&format!("{}i64.const 0\n", indent_str));
                        code.push_str(&format!("{}i64.sub\n", indent_str)); // 0 - expr
                    }
                    UnaryOperator::Not => {
                        code.push_str(&format!("{}i64.eqz\n", indent_str));
                        code.push_str(&format!("{}i64.extend_i32_u\n", indent_str));
                    }
                }
                
                code
            }
            _ => format!("{}i64.const 0  ;; unimplemented expression\n", indent_str),
        }
    }
}