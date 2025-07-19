use crate::compiler::ast::{Stmt, Expr};
use std::collections::HashMap;

pub struct Interpreter;



impl Interpreter {
    pub fn run(stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::Function(func) if func.name == "main" => {
                    Self::run_block(&func.body);
                }
                _ => {}
            }
        }
    }

    fn run_block(stmts: &[Stmt]) {
        let mut env = HashMap::new();

        for stmt in stmts {
            match stmt {
                Stmt::Let(name, expr) => {
                    let val = Self::eval_expr(expr, &env);
                    env.insert(name.clone(), val);
                }
                Stmt::Print(expr) => {
                    let val = Self::eval_expr(expr, &env);
                    println!("{val}");
                }
                _ => {}
            }
        }
    }

    fn eval_expr(expr: &Expr, env: &HashMap<String, i64>) -> i64 {
        match expr {
            Expr::Number(n) => *n,
            Expr::Ident(name) => *env.get(name).expect("Undefined variable"),
            Expr::BinaryOp(lhs, op, rhs) => {
                let left = Self::eval_expr(lhs, env);
                let right = Self::eval_expr(rhs, env);
                match op.as_str() {
                    "+" => left + right,
                    "-" => left - right,
                    "*" => left * right,
                    "/" => left / right,
                    "%" => left % right,
                    _ => panic!("Unknown operator: {op}"),
                }
            }


        Expr::BinaryOp(left, op, right) => {
            let l = Self::eval_expr(left, env);
            let r = Self::eval_expr(right, env);
            match op.as_str() {
                "+" => l + r,
                "-" => l - r,
                "*" => l * r,
                "/" => l / r,
                _ => panic!("Unknown operator {}", op),
            }
        }

    }
}

}
