use std::process::exit;
use std::rc::Rc;
use crate::lang::parsing::error::parse_err;
use crate::lang::parsing::ast::{BExpr, Expr, Stmt};
use crate::lang::parsing::ast::Expr::{Assignment, Grouping, Literal, Logical, Variable};
use crate::lang::parsing::ast::Stmt::Block;
use crate::lang::parsing::token::{LiteralValue, Token, TokenType};
use crate::lang::parsing::token::TokenType::*;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

macro_rules! binary {
    ($self: ident,[$($operator:expr),*], $builder: ident) => {
        let mut current_expr: BExpr = $self.$builder();
        while $self.match_next_multi(&vec![$($operator,)*]) {
            current_expr = Box::from(Expr::Binary((*$self.peek_bw(1)).clone(), current_expr, $self.$builder()));
        }
        return current_expr;
    }
}

impl<'a> Parser<'a> {

    pub fn parse(tokens: &Vec<Token>) -> Vec<Stmt> {
        let mut parser = Parser {
            tokens,
            current: 0
        };
        parser.program()
    }

    fn program(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration());
        }
        self.consume(Eof, "Expected EOF char at the end of file");
        statements
    }

    fn declaration(&mut self) -> Stmt {
        if self.match_next(Var) {
            return self.var_declaration()
        }
        if self.match_next(Fun) {
            return self.fun_declaration()
        }
        self.statement()
    }

    fn statement(&mut self) -> Stmt {
        if self.match_next(If) {
            return self.if_statement();
        }
        if self.match_next(While) {
            return self.while_statement();
        }
        if self.match_next(For) {
            return self.for_statement();
        }
        if self.match_next(Loop) {
            return self.loop_statement();
        }
        if self.match_next(Break) {
            return self.break_statement();
        }
        if self.match_next(Continue) {
            return self.continue_statement();
        }
        if self.match_next(Return) {
            return self.return_statement();
        }
        if self.match_next(LeftBrace) {
            return self.block();
        }
        self.expression_statement()
    }

    fn if_statement(&mut self) -> Stmt {
        self.consume(LeftParen, "Expected '(' after if.");
        let condition = self.expression();
        self.consume(RightParen, "Expected ')' after if condition.");
        let if_branch = self.statement();

        if self.match_next(Else) {
            let else_branch = self.statement();
            return Stmt::If(condition, Box::from(if_branch), Some(Box::from(else_branch)));
        }
        Stmt::If(condition, Box::from(if_branch), None)
    }

    fn loop_statement(&mut self) -> Stmt {
        let inner_stmt = self.declaration();
        Stmt::Loop(None, Box::from(inner_stmt), None)
    }

    fn while_statement(&mut self) -> Stmt {
        self.consume(LeftParen, "Expected '(' after while.");
        let condition = self.expression();
        self.consume(RightParen, "Expected ')' after while condition.");
        let inner_stmt = self.declaration();

        Stmt::Loop(Some(condition), Box::from(inner_stmt), None)
    }

    fn return_statement(&mut self) -> Stmt {
        let tok = self.peek_bw(1);
        let mut expr: Option<BExpr> = None;
        if !self.cmp_tok(&Semicolon) {
            expr = Some(self.expression());
        }
        self.consume(Semicolon, "Expected ';' after return value.");

        Stmt::Return(tok.clone(), expr)
    }

    fn for_statement(&mut self) -> Stmt {
        self.consume(LeftParen, "Expected '(' after for.");

        let initializer = if self.match_next(Semicolon) { None } else { Some(self.declaration()) };

        let condition = if self.match_next(Semicolon) { None }
        else {
            let wrapped = self.expression();
            self.consume(Semicolon, "Expected ';' after expression.");
            Some(wrapped)
        };

        let increment = if self.match_next(RightParen) { None }
        else {
            let wrapped = self.expression();
            self.consume(RightParen, "Expected ')' after body.");
            Some(Box::from(Stmt::Expression(wrapped)))
        };

        let inner_stmt = Box::from(self.declaration());

        if initializer.is_none() {
            return Stmt::Loop(condition,inner_stmt, increment);
        }
        Stmt::Block(vec![
            initializer.unwrap(),
            Stmt::Loop(condition, inner_stmt, increment)
        ])
    }

    fn block(&mut self) -> Stmt {
        let mut statements: Vec<Stmt> = vec![];

        while !self.cmp_tok(&RightBrace) && !self.is_at_end() {
            statements.push(self.declaration());
        }

        self.consume(RightBrace, "Expected '}' after block.");

        Stmt::Block(statements)
    }

    fn break_statement(&mut self) -> Stmt {
        let tok = self.peek_bw(1);
        self.consume(Semicolon, "Expected ';' after value");
        Stmt::Break(tok.clone())
    }

    fn continue_statement(&mut self) -> Stmt {
        let tok = self.peek_bw(1);
        self.consume(Semicolon, "Expected ';' after value");
        Stmt::Continue(tok.clone())
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(Semicolon, "Expected ';' after expression");
        Stmt::Expression(expr)
    }

    fn fun_declaration(&mut self) -> Stmt {
        let token = self.consume(Identifier, "Expected identifier after 'fun'").clone();
        self.consume(LeftParen, "Expected '(' after function name");
        let mut parameters: Vec<Token> = vec![];
        if !self.cmp_tok(&RightParen) {
            parameters.push(self.consume(Identifier, "Identifier expected").clone());
            while self.match_next(Comma) {
                parameters.push(self.consume(Identifier, "Identifier expected").clone());
            }
        }
        self.consume(RightParen, "Expected ')' after parameter list");
        self.consume(LeftBrace, "Expected '{' before function body");
        let block = self.block();

        let body = match block {
            Block(stmts) => stmts,
            _ => vec![]
        };

        Stmt::Function(token, parameters, Rc::new(body))
    }

    fn var_declaration(&mut self) -> Stmt {
        let token = self.consume(Identifier, "Expected identifier after 'var'").clone();
        let expr = match self.match_next(Equal) {
            true => self.expression(),
            false => Box::from(Literal(LiteralValue::Nil))
        };
        self.consume(Semicolon, "Expected ';' after expression");
        Stmt::Declaration(token, expr)
    }

    fn expression(&mut self) -> BExpr {
        self.assignment()
    }

    fn assignment(&mut self) -> BExpr {
        let expr = self.or();

        if self.match_next(Equal) {
            let equals = self.peek_bw(1);
            let value = self.assignment();
            match *expr {
                Variable(tok) => {
                    return Box::from(Assignment(tok, value));
                },
                _ => {
                    parse_err("Invalid assignment target", equals.line);
                    exit(1);
                },
            }
        }
        expr
    }

    fn or(&mut self) -> BExpr {
        let expr = self.and();
        if self.match_next(Or) {
            let op = self.peek_bw(1);
            let right = self.and();
            return Box::from(Logical(expr, op.clone(), right));
        }
        expr
    }

    fn and(&mut self) -> BExpr {
        let expr = self.equality();
        if self.match_next(And) {
            let op = self.peek_bw(1);
            let right = self.equality();
            return Box::from(Logical(expr, op.clone(), right));
        }
        expr
    }

    fn equality(&mut self) -> BExpr {
        binary!(self, [BangEqual, EqualEqual], comparison);
    }

    fn comparison(&mut self) -> BExpr {
        binary!(self, [Greater, GreaterEqual, Less, LessEqual], term);
    }

    fn term(&mut self) -> BExpr {
        binary!(self, [Plus, Minus], factor);
    }

    fn factor(&mut self) -> BExpr {
        binary!(self, [Star, Slash], unary);
    }

    fn unary(&mut self) -> BExpr {
        if self.match_next_multi(&vec![Minus, Bang]) {
            return Box::from(Expr::Unary((*self.peek_bw(1)).clone(), self.unary()));
        }
        self.call()
    }

    fn finish_call(&mut self, callee: BExpr) -> BExpr {
        let mut arguments: Vec<BExpr> = vec![];
        if !self.cmp_tok(&RightParen) {
            arguments.push(self.expression());
            while self.match_next(Comma) {
                arguments.push(self.expression());
            }
        }
        let paren = self.consume(RightParen, "Expected ')' after argument list.");

        Box::from(Expr::Call(callee, paren.clone(), arguments))
    }

    fn call(&mut self) -> BExpr {
        let mut expr = self.primary();

        loop {
            if self.match_next(LeftParen) {
                expr = self.finish_call(expr);
            }
            else {
                break;
            }
        }

        expr
    }

    fn primary(&mut self) -> BExpr {
        let tok = self.peek_bw(0);
        self.current += 1;
        match &tok.tok_type {
            True => Box::from(Literal(LiteralValue::Bool(true))),
            False => Box::from(Literal(LiteralValue::Bool(false))),
            Nil => Box::from(Literal(LiteralValue::Nil)),
            Str | Num => Box::from(Literal(tok.literal.clone().unwrap())),
            LeftParen => {
                let expr = self.expression();
                self.consume(RightParen, "Expected ')' after expression");
                Box::from(Grouping(expr))
            },
            Identifier => Box::from(Variable(tok.clone())),
            _ => {
                parse_err(&format!("Unexpected token '{}'", tok.lexeme.clone().unwrap()), tok.line);
                exit(1);
            },
        }
    }

    fn consume(&mut self, expected_tok_type: TokenType, error_msg: &str) -> &Token {
        if self.cmp_tok(&expected_tok_type) {
            return self.advance();
        }
        parse_err(error_msg, self.peek_bw(0).line);
        exit(1);
    }

    fn advance(&mut self) -> &'a Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.peek_bw(1)
    }

    fn is_at_end(&self) -> bool {
        self.cmp_tok(&Eof)
    }

    fn peek_bw(&self, offset: usize) -> &'a Token {
        &self.tokens[self.current - offset]
    }

    fn cmp_tok(&self, t: &TokenType) -> bool {
        let current = self.peek_bw(0);
        current.tok_type == *t
    }

    fn match_next(&mut self, t: TokenType) -> bool {
        if self.cmp_tok(&t) {
            self.advance();
            return true;
        }
        false
    }

    fn match_next_multi(&mut self, types: &Vec<TokenType>) -> bool {
        for t in types {
            if self.cmp_tok(t) {
                self.advance();
                return true;
            }
        }
        false
    }
}