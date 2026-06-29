use crate::{
    ast::expr::{Expr, ExprKind},
    diagnostic::{TolDiagnostic, error::TolError},
    module::Module,
    prelude::TolResult,
    token::{Associativity, Token, TokenKind},
};

pub struct Parser<'m> {
    tokens: Vec<Token>,
    current: usize,
    modul: &'m mut Module,
}

impl<'m> Parser<'m> {
    pub fn new(tokens: Vec<Token>, modul: &'m mut Module) -> Self {
        Self {
            tokens,
            current: 0,
            modul,
        }
    }

    pub fn run(&mut self) {
        if let Err(diag) = self.parse_expression(0) {
            self.modul.add_diagnostic(diag);
        };
    }

    fn parse_expression(&mut self, precedence: u8) -> TolResult<Expr> {
        let mut left = self.nud()?;

        while !self.is_at_end() && self.peek().kind().precedence() > precedence {
            let op = self.advance().clone();
            left = self.led(op, left)?;
        }

        Ok(left)
    }

    fn nud(&mut self) -> TolResult<Expr> {
        match self.peek().kind() {
            TokenKind::IntLiteral => {
                let token = self.advance();
                let span = token.span().clone();
                Ok(Expr::new(ExprKind::Integer(token.clone()), span))
            }
            TokenKind::FloatLiteral => {
                let token = self.advance();
                let span = token.span().clone();
                Ok(Expr::new(ExprKind::Float(token.clone()), span))
            }
            TokenKind::Identifier => {
                let token = self.advance();
                let span = token.span().clone();
                Ok(Expr::new(ExprKind::Identifier(token.clone()), span))
            }
            TokenKind::LParen => {
                self.advance(); // Consume `(`
                let expr = self.parse_expression(0)?;
                self.consume(TokenKind::RParen, ")")?;
                Ok(expr)
            }
            _ => Err(TolDiagnostic::new_error(
                TolError::InvalidStartOfAnExpression {
                    span: self.advance().span().clone().into(),
                },
            )),
        }
    }

    fn led(&mut self, op: Token, left: Expr) -> TolResult<Expr> {
        let precedence = match op.kind().assoc() {
            Associativity::Left => op.kind().precedence(),
            Associativity::Right => op.kind().precedence() + 1,
        };

        match op.kind() {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                let right = self.parse_expression(precedence)?;
                let span = left.span().start..right.span().end;

                Ok(Expr::new(
                    ExprKind::Binary {
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                        op,
                    },
                    span,
                ))
            }
            _ => unreachable!(),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_next(&self) -> &Token {
        &self.tokens[self.current + 1]
    }

    fn advance(&mut self) -> &Token {
        self.current += 1;
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, expected_kind: TokenKind, expected_lexeme: &str) -> TolResult<&Token> {
        if self.peek().kind() == &expected_kind {
            return Ok(self.advance());
        }

        Err(TolDiagnostic::new_error(TolError::UnexpectedToken {
            token: self.peek().lexeme().to_string(),
            expected: expected_lexeme.to_string(),
            span: self.peek().span().clone().into(),
        }))
    }

    pub fn is_at_end(&self) -> bool {
        self.tokens[self.current].kind() == &TokenKind::Eof
    }
}
