use crate::lang::token::Keyword;
use crate::lang::token::Keyword::*;
use crate::lang::token::SqlKeyword::*;
use crate::lang::token::Symbol::*;
use crate::lang::token::TokenType::*;
use crate::lang::token::{Span, Token, TokenType};
use crate::runtime::types::RV;
use crate::{kw, skw, sym};
use std::rc::Rc;

use super::ast::expr::Expr;
use super::ast::expr::ExprId;
use super::ast::sql::SelectCore;
use super::ast::sql::SqlCompoundOperator;
use super::ast::sql::SqlDistinct;
use super::ast::sql::SqlExpr;
use super::ast::sql::SqlFrom;
use super::ast::sql::SqlProjection;
use super::ast::sql::SqlSelect;
use super::ast::sql::SqlTableSubquery;
use super::ast::stmt::Stmt;
use super::ast::stmt::StmtId;
use super::ast::ParserArena;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    arena: ParserArena,
}

pub struct Parsed {
    pub program: StmtId,
    pub arena: Rc<ParserArena>,
}

impl Parsed {
    pub fn new(program: StmtId, arena: Rc<ParserArena>) -> Parsed {
        Parsed { program, arena }
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { token: Token },
    MissingToken { token: Token, expected: TokenType },
    InvalidAssignmentTarget { left: Token },
}

type ParseResult<T> = Result<T, ParseError>;

macro_rules! binary {
    ($self: ident, [$($operator:expr),*], $builder: ident) => {
        let mut current_expr: ExprId = $self.$builder()?;
        while $self.match_next_multi(&vec![$($operator,)*]) {
            let token = (*$self.peek_bw(1)).clone();
            let left = current_expr;
            let right = $self.$builder()?;

            current_expr = $self.arena.expression(Expr::Binary {
                left,
                symbol: token.tok_type,
                right,
                span: Span { start: 0, end: 0, line: 0, line_end: 0 }
            });
        }
        return Ok(current_expr);
    }
}

// a macro for repeating match_next pattern
macro_rules! match_next {
    ($self: ident, $t: expr, $callee: ident) => {
        if $self.match_next($t) {
            return $self.$callee();
        }
    };
}

macro_rules! optional_with_expected {
    ($self: ident, $optional: expr, $expected: expr) => {
        if $self.match_next($optional) {
            let token = $self.expected($expected);
            Some(token.unwrap().clone())
        } else if $self.match_next($expected) {
            let token = $self.peek_bw(1);
            Some(token.clone())
        } else {
            None
        }
    };
}

impl<'a> Parser<'a> {
    pub fn parse(tokens: &Vec<Token>) -> ParseResult<Parsed> {
        let arena = ParserArena::new();
        let mut parser = Parser {
            tokens,
            current: 0,
            arena,
        };
        let program = parser.program()?;
        Ok(Parsed::new(program, Rc::new(parser.arena)))
    }

    fn program(&mut self) -> ParseResult<StmtId> {
        let mut statements: Vec<StmtId> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.expected(Eof)?;
        Ok(self.arena.statement(Stmt::Program {
            stmts: statements,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn declaration(&mut self) -> ParseResult<StmtId> {
        match_next!(self, kw!(Var), var_declaration);
        self.statement()
    }

    fn statement(&mut self) -> ParseResult<StmtId> {
        match_next!(self, kw!(If), if_statement);
        match_next!(self, kw!(While), while_statement);
        match_next!(self, kw!(For), for_statement);
        match_next!(self, kw!(Loop), loop_statement);
        match_next!(self, kw!(Break), break_statement);
        match_next!(self, kw!(Continue), continue_statement);
        match_next!(self, kw!(Return), return_statement);
        match_next!(self, sym!(LeftBrace), block);
        self.expression_statement()
    }

    fn if_statement(&mut self) -> ParseResult<StmtId> {
        self.expected(sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(sym!(RightParen))?;
        let if_branch = self.statement()?;

        if self.match_next(kw!(Else)) {
            let else_branch = self.statement()?;
            return Ok(self.arena.statement(Stmt::If {
                condition,
                body: if_branch,
                r#else: Some(else_branch),
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    line_end: 0,
                },
            }));
        }
        Ok(self.arena.statement(Stmt::If {
            condition,
            body: if_branch,
            r#else: None,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn loop_statement(&mut self) -> ParseResult<StmtId> {
        let inner_stmt = self.declaration()?;
        Ok(self.arena.statement(Stmt::Loop {
            condition: None,
            body: inner_stmt,
            post: None,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn while_statement(&mut self) -> ParseResult<StmtId> {
        self.expected(sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(sym!(RightParen))?;
        let inner_stmt = self.declaration()?;

        Ok(self.arena.statement(Stmt::Loop {
            condition: Some(condition),
            body: inner_stmt,
            post: None,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn return_statement(&mut self) -> ParseResult<StmtId> {
        let token = self.peek_bw(1);
        let mut expr: Option<ExprId> = None;
        if !self.cmp_tok(&sym!(Semicolon)) {
            expr = Some(self.expression()?);
        }
        self.expected(sym!(Semicolon))?;

        if expr.is_none() {
            return Ok(self.arena.statement(Stmt::Return {
                span: token.span,
                expr: None,
            }));
        }

        Ok(self.arena.statement(Stmt::Return {
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
            expr,
        }))
    }

    fn for_statement(&mut self) -> ParseResult<StmtId> {
        self.expected(sym!(LeftParen))?;

        let initializer = if self.match_next(sym!(Semicolon)) {
            None
        } else {
            Some(self.declaration()?)
        };

        let condition = if self.match_next(sym!(Semicolon)) {
            None
        } else {
            let wrapped = self.expression()?;
            self.expected(sym!(Semicolon))?;
            Some(wrapped)
        };

        let increment = if self.match_next(sym!(RightParen)) {
            None
        } else {
            let wrapped = self.expression()?;
            self.expected(sym!(RightParen))?;
            Some(self.arena.statement(Stmt::Expression {
                expr: wrapped,
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    line_end: 0,
                },
            }))
        };

        let inner_stmt = self.declaration()?;

        if initializer.is_none() {
            return Ok(self.arena.statement(Stmt::Loop {
                condition,
                body: inner_stmt,
                post: increment,
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    line_end: 0,
                },
            }));
        }
        let loop_stmt = self.arena.statement(Stmt::Loop {
            condition,
            body: inner_stmt,
            post: increment,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        });
        Ok(self.arena.statement(Stmt::Block {
            stmts: vec![initializer.unwrap(), loop_stmt],
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn block(&mut self) -> ParseResult<StmtId> {
        let mut statements: Vec<StmtId> = vec![];

        while !self.cmp_tok(&sym!(RightBrace)) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.expected(sym!(RightBrace))?;

        Ok(self.arena.statement(Stmt::Block {
            stmts: statements,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn break_statement(&mut self) -> ParseResult<StmtId> {
        let tok = self.peek_bw(1);
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Break { span: tok.span }))
    }

    fn continue_statement(&mut self) -> ParseResult<StmtId> {
        let tok = self.peek_bw(1);
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Continue { span: tok.span }))
    }

    fn expression_statement(&mut self) -> ParseResult<StmtId> {
        let expr = self.expression()?;
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Expression {
            expr,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn var_declaration(&mut self) -> ParseResult<StmtId> {
        let token = self.expected(Identifier { dollar: true })?.clone();
        let expr = match self.match_next(sym!(Equal)) {
            true => self.expression()?,
            false => self.arena.expression(Expr::Literal {
                value: RV::Undefined,
                raw: "undefined".to_string(),
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    line_end: 0,
                },
            }),
        };
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Declaration {
            dst: token,
            expr,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn fun_declaration(&mut self) -> ParseResult<ExprId> {
        let token = if self.cmp_tok(&Identifier { dollar: false }) {
            Some(self.expected(Identifier { dollar: false })?.clone())
        } else {
            None
        };

        self.expected(sym!(LeftParen))?;
        let mut parameters: Vec<Token> = vec![];
        if !self.cmp_tok(&sym!(RightParen)) {
            let p = self.expected(Identifier { dollar: true })?;
            parameters.push(p.clone());
            while self.match_next(sym!(Comma)) {
                let q = self.expected(Identifier { dollar: true })?;
                parameters.push(q.clone());
            }
        }
        self.expected(sym!(RightParen))?;
        self.expected(sym!(LeftBrace))?;
        let bidx = self.block()?;

        let block = self.arena.get_statement(bidx);

        let body: Vec<StmtId> = match block {
            Stmt::Block { stmts, span: _ } => stmts.clone(),
            _ => vec![],
        };

        Ok(self.arena.expression(Expr::Function {
            name: token,
            parameters,
            body: Rc::new(body),
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
        }))
    }

    fn expression(&mut self) -> ParseResult<ExprId> {
        match_next!(self, kw!(Fun), fun_declaration);
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<ExprId> {
        let expr = self.or()?;

        if self.match_next(sym!(Equal)) {
            let value = self.assignment()?;
            match self.arena.get_expression(expr) {
                Expr::Variable { name, span: _ } => {
                    return Ok(self.arena.expression(Expr::Assignment {
                        dst: name.clone(),
                        expr: value,
                        span: Span {
                            start: 0,
                            end: 0,
                            line: 0,
                            line_end: 0,
                        },
                    }));
                }
                _ => {
                    return Err(ParseError::InvalidAssignmentTarget {
                        left: self.peek_bw(3).clone(),
                    });
                }
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<ExprId> {
        let expr = self.and()?;
        if self.match_next(kw!(Keyword::Or)) {
            let op = self.peek_bw(1);
            let right = self.and()?;
            return Ok(self.arena.expression(Expr::Logical {
                left: expr,
                symbol: op.tok_type.clone(),
                right,
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    line_end: 0,
                },
            }));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<ExprId> {
        let expr = self.equality()?;
        if self.match_next(kw!(Keyword::And)) {
            let op = self.peek_bw(1);
            let right = self.equality()?;
            return Ok(self.arena.expression(Expr::Logical {
                left: expr,
                symbol: op.tok_type.clone(),
                right,
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    line_end: 0,
                },
            }));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<ExprId> {
        binary!(self, [sym!(BangEqual), sym!(EqualEqual)], comparison);
    }

    fn comparison(&mut self) -> ParseResult<ExprId> {
        binary!(
            self,
            [
                sym!(Greater),
                sym!(GreaterEqual),
                sym!(Less),
                sym!(LessEqual)
            ],
            term
        );
    }

    fn term(&mut self) -> ParseResult<ExprId> {
        binary!(self, [sym!(Plus), sym!(Minus)], factor);
    }

    fn factor(&mut self) -> ParseResult<ExprId> {
        binary!(self, [sym!(Star), sym!(Slash)], unary);
    }

    fn unary(&mut self) -> ParseResult<ExprId> {
        if self.match_next_multi(&vec![sym!(Minus), sym!(Bang)]) {
            let token = (*self.peek_bw(1)).clone();
            let unary = self.unary()?;
            return Ok(self.arena.expression(Expr::Unary {
                symbol: token.tok_type,
                expr: unary,
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    line_end: 0,
                },
            }));
        }
        self.select()
    }

    fn select(&mut self) -> ParseResult<ExprId> {
        if !self.cmp_tok(&skw!(Select)) {
            return self.call();
        }
        let core = self.select_core()?;
        let mut compounds: Vec<(SqlCompoundOperator, SelectCore)> = vec![];
        while self.match_next_multi(&vec![skw!(Union), skw!(Intersect), skw!(Except)]) {
            let op = self.peek_bw(1);
            let compound_op = if op.tok_type == skw!(Union) && self.match_next(skw!(All)) {
                SqlCompoundOperator::UnionAll
            } else {
                match op.tok_type {
                    SqlKeyword(Union) => SqlCompoundOperator::Union,
                    SqlKeyword(Intersect) => SqlCompoundOperator::Intersect,
                    SqlKeyword(Except) => SqlCompoundOperator::Except,
                    _ => {
                        return Err(ParseError::UnexpectedToken { token: op.clone() });
                    }
                }
            };
            let secondary_core = self.select_core()?;
            compounds.push((compound_op, secondary_core))
        }
        Ok(self.arena.expression(Expr::Select {
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
            query: SqlSelect {
                core,
                compound: compounds,
                order_by: None, // TODO(vck)
                limit: None,    // TODO(vck)
                offset: None,   // TODO(vck)
            },
        }))
    }

    fn select_core(&mut self) -> ParseResult<SelectCore> {
        self.expected(skw!(Select))?;
        let distinct = if self.match_next(skw!(Distinct)) {
            SqlDistinct::Distinct
        } else {
            SqlDistinct::All
        };
        /* else if self.match_next(skw!(All)) {
            SqlDistinct::All
        }*/

        Ok(SelectCore {
            distinct,
            projection: self.sql_projection(),
            from: self.sql_from()?,
            r#where: self.sql_where()?,
            group_by: None, // TODO(vck)
            having: None,   // TODO(vck)
        })
    }

    fn sql_projection(&mut self) -> Vec<SqlProjection> {
        let mut projections: Vec<SqlProjection> = vec![];
        loop {
            if self.match_next(sym!(Star)) {
                projections.push(SqlProjection::All);
            } else {
                let expr = self.expression().unwrap();
                let alias: Option<Token> =
                    optional_with_expected!(self, skw!(As), Identifier { dollar: false });
                projections.push(SqlProjection::Complex {
                    expr: SqlExpr::Default(expr),
                    alias,
                });
            }
            if !self.match_next(sym!(Comma)) {
                break;
            }
        }
        // TODO(vck): Add support for table selectors
        projections
    }

    fn sql_from(&mut self) -> ParseResult<Option<SqlFrom>> {
        if self.match_next(skw!(From)) {
            let token = self.expected(Identifier { dollar: false });
            return Ok(Some(SqlFrom::TableSubquery(vec![
                SqlTableSubquery::Simple {
                    namespace: None,
                    table: token.unwrap().clone(),
                    alias: None,
                },
            ])));
        }
        // TODO(vck): Joins
        Ok(None)
    }

    fn sql_where(&mut self) -> ParseResult<Option<SqlExpr>> {
        if self.match_next(skw!(Where)) {
            let expr = self.expression()?;
            return Ok(Some(SqlExpr::Default(expr)));
        }
        Ok(None)
    }

    fn finish_call(&mut self, callee: ExprId) -> ParseResult<ExprId> {
        let mut arguments: Vec<ExprId> = vec![];
        if !self.cmp_tok(&sym!(RightParen)) {
            arguments.push(self.expression()?);
            while self.match_next(sym!(Comma)) {
                arguments.push(self.expression()?);
            }
        }
        let paren = self.expected(sym!(RightParen))?.clone();

        Ok(self.arena.expression(Expr::Call {
            callee,
            span: Span {
                start: 0,
                end: 0,
                line: 0,
                line_end: 0,
            },
            args: arguments,
        }))
    }

    fn call(&mut self) -> ParseResult<ExprId> {
        let mut expr = self.primary()?;

        loop {
            if self.match_next(sym!(LeftParen)) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> ParseResult<ExprId> {
        let tok = self.peek_bw(0);
        self.current += 1;
        match &tok.tok_type {
            True => Ok(self.arena.expression(Expr::Literal {
                value: RV::Bool(true),
                raw: "true".to_string(),
                span: tok.span,
            })),
            False => Ok(self.arena.expression(Expr::Literal {
                value: RV::Bool(false),
                raw: "false".to_string(),
                span: tok.span,
            })),
            TokenType::Null => Ok(self.arena.expression(Expr::Literal {
                value: RV::Null,
                raw: "null".to_string(),
                span: tok.span,
            })),
            Str | Num => Ok(self.arena.expression(Expr::Literal {
                value: tok.literal.clone().unwrap(),
                raw: tok.lexeme.clone().unwrap(),
                span: tok.span,
            })),
            Identifier { dollar: _ } => Ok(self.arena.expression(Expr::Variable {
                name: tok.clone(),
                span: tok.span,
            })),
            Symbol(LeftParen) => {
                let expr = self.expression()?;
                self.expected(sym!(RightParen))?;
                Ok(self.arena.expression(Expr::Grouping {
                    span: Span {
                        start: 0,
                        end: 0,
                        line: 0,
                        line_end: 0,
                    },
                    expr,
                }))
            }
            _ => Err(ParseError::UnexpectedToken { token: tok.clone() }),
        }
    }

    fn expected(&mut self, expected_tok_type: TokenType) -> ParseResult<&Token> {
        if self.cmp_tok(&expected_tok_type) {
            return Ok(self.advance());
        };
        let prev_token = self.peek_bw(1);
        Err(ParseError::MissingToken {
            token: prev_token.clone(),
            expected: expected_tok_type,
        })
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

#[cfg(test)]
mod test {

    use assert_json_diff::assert_json_eq;
    use serde_json::{json, Value};

    use crate::lang::{scanner::Scanner, token::Token};

    use super::*;

    fn get_tokens(source: &str) -> Vec<Token> {
        return Scanner::scan(source).unwrap();
    }

    fn compare_parsed_to_expected(source: &str, expected: Value) {
        let tokens = get_tokens(source);
        let mut parsed = Parser::parse(&tokens).unwrap();
        let actual = parsed.to_json();
        assert_json_eq!(actual, expected);
    }

    #[test]
    fn test_parse_literal_expression() {
        compare_parsed_to_expected(
            "1;",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Literal",
                            "value": "Num(1.0)",
                            "raw": "1"
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn test_parse_unary_expression() {
        compare_parsed_to_expected(
            "-1;",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Unary",
                            "operator": {
                                "Symbol": "Minus"
                            },
                            "value": {
                                "type": "Expr::Literal",
                                "value": "Num(1.0)",
                                "raw": "1"
                            }
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn test_parse_binary_expression() {
        compare_parsed_to_expected(
            "1 + 2;",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Binary",
                            "left": {
                                "type": "Expr::Literal",
                                "value": "Num(1.0)",
                                "raw": "1"
                            },
                            "operator": {
                                "Symbol": "Plus"
                            },
                            "right": {
                                "type": "Expr::Literal",
                                "value": "Num(2.0)",
                                "raw": "2"
                            }
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn test_parse_grouping_expression() {
        compare_parsed_to_expected(
            "(1 + 2) * (3 / (4 - 7));",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Binary",
                            "left": {
                                "type": "Expr::Grouping",
                                "value": {
                                    "type": "Expr::Binary",
                                    "left": {
                                        "raw": "1",
                                        "type": "Expr::Literal",
                                        "value": "Num(1.0)",
                                    },
                                    "operator": {
                                        "Symbol": "Plus"
                                    },
                                    "right": {
                                        "raw": "2",
                                        "type": "Expr::Literal",
                                        "value": "Num(2.0)",
                                    }
                                }
                            },
                            "operator": {
                                "Symbol": "Star"
                            },
                            "right": {
                                "type": "Expr::Grouping",
                                "value": {
                                    "type": "Expr::Binary",
                                    "left": {
                                        "raw": "3",
                                        "type": "Expr::Literal",
                                        "value": "Num(3.0)",
                                    },
                                    "operator": {
                                        "Symbol": "Slash"
                                    },
                                    "right": {
                                        "type": "Expr::Grouping",
                                        "value": {
                                            "type": "Expr::Binary",
                                            "left": {
                                                "raw": "4",
                                                "type": "Expr::Literal",
                                                "value": "Num(4.0)",
                                            },
                                            "operator":  {
                                                "Symbol": "Minus"
                                            },
                                            "right": {
                                                "raw": "7",
                                                "type": "Expr::Literal",
                                                "value": "Num(7.0)",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn test_parse_variable_expression() {
        compare_parsed_to_expected(
            "a;",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Variable",
                            "value": "a",
                        }
                    }
                ]
            }),
        );
    }
}
