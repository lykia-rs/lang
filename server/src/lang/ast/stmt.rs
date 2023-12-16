use crate::lang::token::{Span, Spanned, Token};

use super::expr::ExprId;

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Program {
        stmts: Vec<StmtId>,
        span: Span,
    },
    Expression {
        expr: ExprId,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Block {
        stmts: Vec<StmtId>,
        span: Span,
    },
    Declaration {
        dst: Token,
        expr: ExprId,
        span: Span,
    },
    If {
        condition: ExprId,
        body: StmtId,
        r#else: Option<StmtId>,
        span: Span,
    },
    Loop {
        condition: Option<ExprId>,
        body: StmtId,
        post: Option<StmtId>,
        span: Span,
    },
    Return {
        expr: Option<ExprId>,
        span: Span,
    },
}

impl Spanned for Stmt {
    fn get_span(&self) -> Span {
        match self {
            Stmt::Program { span, .. } => *span,
            Stmt::Expression { span, .. } => *span,
            Stmt::Break { span, .. } => *span,
            Stmt::Continue { span, .. } => *span,
            Stmt::Block { span, .. } => *span,
            Stmt::Declaration { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::Loop { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StmtId(pub usize);