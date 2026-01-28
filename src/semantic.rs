use crate::parser::{AstNode, BinOp, Parameter};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct VarInfo {
    is_consumed: bool,
    borrow_count: usize,
    is_mutable: bool,
    declared_line: usize,
    var_type: String,
}

pub struct SemanticAnalyzer<'a> {
    filename: &'a str,
    symbol_table: Vec<HashMap<String, VarInfo>>,
    current_line: usize,
    in_loop: bool,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(filename: &'a str) -> Self {
        SemanticAnalyzer {
            filename,
            symbol_table: vec![HashMap::new()],
            current_line: 0,
            in_loop: false,
        }
    }

    fn is_copy_type(&self, name: &str) -> bool {
        if let Some(info) = self.lookup_variable(name) {
            matches!(info.var_type.as_str(), "int" | "bool" | "char" | "string")
        } else {
            false
        }
    }

    pub fn analyze(&mut self, ast: &AstNode) -> Result<(), String> {
        self.visit(ast)
    }

    fn visit(&mut self, node: &AstNode) -> Result<(), String> {
        match node {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.visit(node)?;
                }
                Ok(())
            }

            AstNode::FunctionDef { params, body, .. } => {
                self.push_scope();

                for param in params {
                    self.declare_variable(
                        &param.name,
                        !param.is_reference,
                        param.param_type.clone(),
                        0,
                    );
                }

                self.visit(body)?;
                self.pop_scope();
                Ok(())
            }

            AstNode::LetBinding { mutable, name, value, type_annotation } => {
                self.visit(value)?;

                if let AstNode::Identifier(var_name) = value.as_ref() {
                    self.check_not_consumed(var_name)?;
                    self.consume_variable(var_name)?;
                }

                let var_type = type_annotation.clone().unwrap_or_else(|| {
                    self.infer_type(value)
                });

                self.declare_variable(name, *mutable, var_type, self.current_line);
                Ok(())
            }

            AstNode::Assignment { name, value } => {
                self.check_not_consumed(name)?;
                self.check_is_mutable(name)?;
                self.check_not_borrowed(name)?;
                self.visit(value)?;

                if let AstNode::Identifier(var_name) = value.as_ref() {
                    self.check_not_consumed(var_name)?;
                    self.consume_variable(var_name)?;
                }

                Ok(())
            }

            AstNode::Block(statements) => {
                self.push_scope();
                for stmt in statements {
                    self.visit(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }

            AstNode::If { condition, then_block, else_block } => {
                self.visit(condition)?;
                self.visit(then_block)?;
                if let Some(else_block) = else_block {
                    self.visit(else_block)?;
                }
                Ok(())
            }

            AstNode::While { condition, body } => {
                self.visit(condition)?;
                let was_in_loop = self.in_loop;
                self.in_loop = true;
                self.visit(body)?;
                self.in_loop = was_in_loop;
                Ok(())
            }

            AstNode::For { variable, iterator, body } => {
                self.visit(iterator)?;
                self.push_scope();

                // Declare loop variable
                self.declare_variable(variable, false, "int".to_string(), self.current_line);

                let was_in_loop = self.in_loop;
                self.in_loop = true;
                self.visit(body)?;
                self.in_loop = was_in_loop;

                self.pop_scope();
                Ok(())
            }

            AstNode::Match { value, arms } => {
                self.visit(value)?;
                for arm in arms {
                    self.visit(&arm.body)?;
                }
                Ok(())
            }

            AstNode::Return(value) => {
                if let Some(value) = value {
                    self.visit(value)?;
                }
                Ok(())
            }

            AstNode::Break => {
                if !self.in_loop {
                    return Err(format!(
                        "{}:{}:{}: Error: 'break' outside of loop",
                        self.filename, self.current_line, 0
                    ));
                }
                Ok(())
            }

            AstNode::Continue => {
                if !self.in_loop {
                    return Err(format!(
                        "{}:{}:{}: Error: 'continue' outside of loop",
                        self.filename, self.current_line, 0
                    ));
                }
                Ok(())
            }

            AstNode::ExpressionStatement(expr) => self.visit(expr),

            AstNode::BinaryOp { left, right, op } => {
                self.visit(left)?;
                self.visit(right)?;

                if matches!(op, BinOp::Add) {
                    if let AstNode::Identifier(var) = left.as_ref() {
                        if self.get_type(var) == Some("string") {
                            self.consume_variable(var)?;
                        }
                    }
                    if let AstNode::Identifier(var) = right.as_ref() {
                        if self.get_type(var) == Some("string") {
                            self.consume_variable(var)?;
                        }
                    }
                }

                Ok(())
            }

            AstNode::UnaryOp { operand, .. } => {
                self.visit(operand)?;
                Ok(())
            }

            AstNode::Identifier(name) => {
                self.check_not_consumed(name)?;
                Ok(())
            }

            AstNode::Reference(expr) => {
                if let AstNode::Identifier(var_name) = expr.as_ref() {
                    self.check_not_consumed(var_name)?;
                    self.borrow_variable(var_name)?;
                }
                self.visit(expr)?;
                Ok(())
            }

            AstNode::Call { args, .. } => {
                for arg in args {
                    self.visit(arg)?;

                    if let AstNode::Identifier(var_name) = arg {
                        self.check_not_consumed(var_name)?;
                        self.consume_variable(var_name)?;
                    }
                    if let AstNode::Reference(ref_expr) = arg {
                        if let AstNode::Identifier(var_name) = ref_expr.as_ref() {
                            self.check_not_consumed(var_name)?;
                        }
                    }
                }
                Ok(())
            }

            AstNode::MethodCall { object, args, .. } => {
                self.visit(object)?;
                for arg in args {
                    self.visit(arg)?;
                }
                Ok(())
            }

            AstNode::MemberAccess { object, .. } => self.visit(object),

            AstNode::Index { array, index } => {
                self.visit(array)?;
                self.visit(index)?;
                Ok(())
            }

            AstNode::ArrayLit(elements) => {
                for elem in elements {
                    self.visit(elem)?;
                }
                Ok(())
            }

            AstNode::StructInit { fields, .. } => {
                for (_, value) in fields {
                    self.visit(value)?;
                }
                Ok(())
            }

            AstNode::EnumValue { value, .. } => {
                if let Some(value) = value {
                    self.visit(value)?;
                }
                Ok(())
            }

            AstNode::StructDef { .. } => Ok(()),
            AstNode::EnumDef { .. } => Ok(()),
            AstNode::ArrayType { .. } => Ok(()),
            AstNode::Number(_) => Ok(()),
            AstNode::Boolean(_) => Ok(()),
            AstNode::Character(_) => Ok(()),
            AstNode::StringLit(_) => Ok(()),
        }
    }

    fn declare_variable(&mut self, name: &str, mutable: bool, var_type: String, line: usize) {
        let scope = self.symbol_table.last_mut().unwrap();
        scope.insert(
            name.to_string(),
            VarInfo {
                is_consumed: false,
                borrow_count: 0,
                is_mutable: mutable,
                declared_line: line,
                var_type,
            },
        );
    }

    fn check_not_consumed(&self, name: &str) -> Result<(), String> {
        // Skip move checking for Copy types
        if self.is_copy_type(name) {
            return Ok(());
        }

        if let Some(info) = self.lookup_variable(name) {
            if info.is_consumed {
                return Err(format!(
                    "{}:{}:{}: Error: use of moved value '{}'
    Note: value moved at line {}, cannot be used again
    Help: Consider borrowing '&{}' to keep ownership in the current scope",
                    self.filename, self.current_line, 0, name, info.declared_line, name
                ));
            }
        }
        Ok(())
    }

    fn check_is_mutable(&self, name: &str) -> Result<(), String> {
        if let Some(info) = self.lookup_variable(name) {
            if !info.is_mutable {
                return Err(format!(
                    "{}:{}:{}: Error: cannot assign to immutable variable '{}'
Help: Consider declaring with 'let mut {}'",
                    self.filename, self.current_line, 0, name, name
                ));
            }
        }
        Ok(())
    }

    fn check_not_borrowed(&self, name: &str) -> Result<(), String> {
        if let Some(info) = self.lookup_variable(name) {
            if info.borrow_count > 0 {
                return Err(format!(
                    "{}:{}:{}: Error: cannot move '{}' while borrowed
Note: {} active borrow(s) exist",
                    self.filename, self.current_line, 0, name, info.borrow_count
                ));
            }
        }
        Ok(())
    }

    fn consume_variable(&mut self, name: &str) -> Result<(), String> {
        // Skip consuming Copy types
        if self.is_copy_type(name) {
            return Ok(());
        }

        for scope in self.symbol_table.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                if info.borrow_count > 0 {
                    return Err(format!(
                        "{}:{}:{}: Error: cannot move '{}' while borrowed",
                        self.filename, self.current_line, 0, name
                    ));
                }
                info.is_consumed = true;
                return Ok(());
            }
        }
        Ok(())
    }

    fn borrow_variable(&mut self, name: &str) -> Result<(), String> {
        for scope in self.symbol_table.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                info.borrow_count += 1;
                return Ok(());
            }
        }
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.symbol_table.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn get_type(&self, name: &str) -> Option<&str> {
        self.lookup_variable(name).map(|info| info.var_type.as_str())
    }

    fn infer_type(&self, expr: &AstNode) -> String {
        match expr {
            AstNode::Number(_) => "int".to_string(),
            AstNode::Boolean(_) => "bool".to_string(),
            AstNode::Character(_) => "char".to_string(),
            AstNode::StringLit(_) => "string".to_string(),
            AstNode::Identifier(name) => {
                self.get_type(name).unwrap_or("unknown").to_string()
            }
            AstNode::BinaryOp { left, .. } => self.infer_type(left),
            AstNode::ArrayLit(elements) => {
                if elements.is_empty() {
                    "[int; 0]".to_string()
                } else {
                    let elem_type = self.infer_type(&elements[0]);
                    format!("[{}; {}]", elem_type, elements.len())
                }
            }
            _ => "unknown".to_string(),
        }
    }

    fn push_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.symbol_table.pop();
    }
}