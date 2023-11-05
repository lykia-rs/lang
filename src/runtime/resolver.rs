use std::process::exit;
use std::rc::Rc;
use rustc_hash::FxHashMap;
use crate::runtime::interpreter::{HaltReason, runtime_err};
use crate::lang::ast::{Expr, Stmt, Visitor, ParserArena, StmtId, ExprId};
use crate::lang::token::Token;
use crate::runtime::types::RV;

pub struct Resolver {
    scopes: Vec<FxHashMap<String, bool>>,
    locals: FxHashMap<usize, usize>,
    arena: Rc<ParserArena>,
}

impl Resolver {
    pub fn new(arena: Rc<ParserArena>) -> Resolver {
        Resolver {
            scopes: vec![],
            locals: FxHashMap::default(),
            arena
        }
    }

    pub fn get_distance(&self, eid: ExprId) -> Option<usize> {
        self.locals.get(&eid).copied()
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_stmts(&mut self, statements: &Vec<StmtId>) {
        for statement in statements {
            self.resolve_stmt(*statement);
        }
    }

    pub fn resolve_stmt(&mut self, statement: StmtId) {
        self.visit_stmt(statement).unwrap();
    }

    pub fn resolve_expr(&mut self, expr: ExprId) {
        self.visit_expr(expr);
    }

    pub fn resolve_local(&mut self, expr: ExprId, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            println!("\tSearching for local variable: {:?}, {:?}, {}, {}, {}", self.scopes, self.locals, name.lexeme.as_ref().unwrap().to_string(), expr, i);
            if self.scopes[i].contains_key(&name.lexeme.as_ref().unwrap().to_string()) {
                self.locals.insert(expr, i);
                println!("\tFound local variable: {:?}, {:?}, {}, {}, {}", self.scopes, self.locals, name.lexeme.as_ref().unwrap().to_string(), expr, i);
                return;
            }
        }
    }

    pub fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let last =  self.scopes.last_mut();
        last.unwrap().insert(name.lexeme.as_ref().unwrap().to_string(), false);
    }

    pub fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let last = self.scopes.last_mut();
        last.unwrap().insert(name.lexeme.as_ref().unwrap().to_string(), true);
        
    }
}

impl Visitor<RV, HaltReason> for Resolver {

    fn visit_expr(&mut self, eidx: ExprId) -> RV {
        let a = Rc::clone(&self.arena);
        let e = a.get_expression(eidx);
        match e {
            Expr::Literal(_) => (),
            Expr::Grouping(expr) => self.resolve_expr(*expr),
            Expr::Unary(_tok, expr) => self.resolve_expr(*expr),
            Expr::Binary(_tok, left, right) => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Variable(tok) => {
                let last_scope = self.scopes.last().unwrap();
                let value = last_scope.get(&tok.lexeme.as_ref().unwrap().to_string());
                
                if !self.scopes.is_empty() &&
                    value.is_some() && *value.unwrap() == false {
                    runtime_err(&"Can't read local variable in its own initializer.", tok.line);
                    exit(1);
                }
                println!("Resolving Expr::Variable: {}", tok.lexeme.as_ref().unwrap());
                self.resolve_local(eidx, tok);
            },
            Expr::Assignment(name, value) => {
                self.resolve_expr(*value);
                println!("Resolving Expr::Assignment: {}", name.lexeme.as_ref().unwrap());
                self.resolve_local(eidx, name);
            },
            Expr::Logical(left, _tok, right) => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            },
            Expr::Call(callee, _paren, arguments) => {
                self.resolve_expr(*callee);

                for argument in arguments {
                    self.resolve_expr(*argument);
                }
            },
            Expr::Select(_) => (),
        };
        RV::Undefined
    }

    fn visit_stmt(&mut self, sidx: StmtId) -> Result<RV, HaltReason> {

        let a = Rc::clone(&self.arena);
        let s = a.get_statement(sidx);
        match s {
            Stmt::Break(_token) |
            Stmt::Continue(_token) => (),
            Stmt::Expression(expr) => {
                self.resolve_expr(*expr);
            },
            Stmt::Declaration(_tok, expr) => {
                self.declare(_tok);
                self.resolve_expr(*expr);
                self.define(_tok);
            },
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_stmts(statements);
                self.end_scope();
            },
            Stmt::If(condition, if_stmt, else_optional) => {
                self.resolve_expr(*condition);
                self.resolve_stmt(*if_stmt);
                self.resolve_stmt(*else_optional.as_ref().unwrap());
            },
            Stmt::Loop(condition, stmt, post_body) => {
                self.resolve_expr(*condition.as_ref().unwrap());
                self.resolve_stmt(*stmt);
                self.resolve_stmt(*post_body.as_ref().unwrap());
            },
            Stmt::Return(_token, expr) => {
                if expr.is_some() {
                    self.resolve_expr(expr.unwrap());
                }
            },
            Stmt::Function(_token, parameters, body) => {
                self.declare(_token);
                self.define(_token);
                self.begin_scope();
                for param in parameters {
                    self.declare(param);
                    self.define(param);
                }
                self.resolve_stmts(body.as_ref());
                self.end_scope();
            },
        }
        Ok(RV::Undefined)
    }
}