use crate::{
    ast::{
        expr::{Expr, ExprKind},
        stmt::Stmt,
    },
    diagnostic::{TolDiagnostic, error::TolError},
    module::Module,
    prelude::TolResult,
    token::{Associativity, Token, TokenKind},
    toltype::TolType,
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
        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(statement) => self.modul.add_statement(statement),
                Err(diag) => self.modul.add_diagnostic(diag),
            }
        }
    }

    fn parse_statement(&mut self) -> TolResult<Stmt> {
        match self.peek().kind() {
            tk if tk == &TokenKind::Identifier && self.peek_next().kind() == &TokenKind::Colon => {
                self.parse_name_declaration()
            }
            // If nothing matched, parse it as an expression statement instead
            _ => {
                let expr = self.parse_expression(0)?;
                let end = self.consume(TokenKind::Semicolon, ";")?.span().end;
                let span = expr.span().start..end;
                Ok(Stmt::new_expression(span, expr))
            }
        }
    }

    fn parse_name_declaration(&mut self) -> TolResult<Stmt> {
        let name = self.advance().clone();
        let start = name.span().start;
        self.advance(); // Consume `:`
        let ty = if self.peek().kind() == &TokenKind::Equal {
            None
        } else {
            Some(self.parse_type()?)
        };

        self.consume(TokenKind::Equal, "=")?;
        let rhs = self.parse_expression(0)?;
        let end = self.consume(TokenKind::Semicolon, ";")?.span().end;

        Ok(Stmt::new_name_declaration(start..end, name, ty, rhs))
    }

    fn parse_type(&mut self) -> TolResult<TolType> {
        let ty = if self.peek().kind() != &TokenKind::Identifier {
            return Err(TolDiagnostic::new_error(TolError::UnexpectedToken {
                token: self.peek().lexeme().to_string(),
                expected: "tipo".to_string(),
                span: self.peek().span().clone().into(),
            }));
        } else {
            self.advance()
        };

        TolType::from_str(ty.lexeme(), ty.span().clone())
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
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Equal => {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(lexeme: &str, kind: TokenKind, span: std::ops::Range<usize>) -> Token {
        Token::new(lexeme.to_string(), kind, span)
    }

    fn eof(at: usize) -> Token {
        Token::new("<EOF>".to_string(), TokenKind::Eof, at..at)
    }

    fn parser_for(tokens: Vec<Token>) -> Parser<'static> {
        let source = Box::leak(String::from("").into_boxed_str());
        let modul = Box::leak(Box::new(Module::new(source)));
        Parser::new(tokens, modul)
    }

    #[test]
    fn parses_precedence_left_associatively() {
        let tokens = vec![
            tok("1", TokenKind::IntLiteral, 0..1),
            tok("+", TokenKind::Plus, 2..3),
            tok("2", TokenKind::IntLiteral, 4..5),
            tok("*", TokenKind::Star, 6..7),
            tok("3", TokenKind::IntLiteral, 8..9),
            eof(9),
        ];

        let mut parser = parser_for(tokens);
        let expr = parser.parse_expression(0).unwrap();

        assert_eq!(expr.to_string(), "(1 + (2 * 3))");
        assert_eq!(expr.span(), &(0..9));
    }

    #[test]
    fn parses_parenthesized_expression() {
        let tokens = vec![
            tok("(", TokenKind::LParen, 0..1),
            tok("1", TokenKind::IntLiteral, 1..2),
            tok("+", TokenKind::Plus, 3..4),
            tok("2", TokenKind::IntLiteral, 5..6),
            tok(")", TokenKind::RParen, 6..7),
            eof(7),
        ];

        let mut parser = parser_for(tokens);
        let expr = parser.parse_expression(0).unwrap();

        assert_eq!(expr.to_string(), "(1 + 2)");
        assert_eq!(expr.span(), &(1..6));
    }

    #[test]
    fn reports_invalid_expression_start() {
        let tokens = vec![tok("+", TokenKind::Plus, 0..1), eof(1)];
        let mut parser = parser_for(tokens);

        let err = parser.parse_expression(0).unwrap_err();

        match err {
            TolDiagnostic::Error(TolError::InvalidStartOfAnExpression { span }) => {
                assert_eq!(span, (0..1).into());
            }
            other => panic!("unexpected diagnostic: {:?}", other),
        }
    }

    #[test]
    fn consumes_expected_closing_paren() {
        let tokens = vec![
            tok("(", TokenKind::LParen, 0..1),
            tok("1", TokenKind::IntLiteral, 1..2),
            tok("+", TokenKind::Plus, 2..3),
            tok("2", TokenKind::IntLiteral, 3..4),
            eof(4),
        ];

        let mut parser = parser_for(tokens);
        let err = parser.parse_expression(0).unwrap_err();

        match err {
            TolDiagnostic::Error(TolError::UnexpectedToken {
                token,
                expected,
                span,
            }) => {
                assert_eq!(token, "<EOF>");
                assert_eq!(expected, ")");
                assert_eq!(span, (4..4).into());
            }
            other => panic!("unexpected diagnostic: {:?}", other),
        }
    }
}
