use std::{collections::HashMap, iter::Peekable, mem, str::Chars};

use crate::{
    diagnostic::TolDiagnostic,
    module::Module,
    prelude::Span,
    token::{Token, TokenKind},
};

pub struct Lexer<'src> {
    source: &'src str,
    source_iter: Peekable<Chars<'src>>,
    start: usize,
    current: usize,
    bracket_depth: u8,
    tokens: Vec<Token<'src>>,
    keywords: HashMap<&'static str, TokenKind>,
    diagnostics: Vec<TolDiagnostic>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        use TokenKind::*;
        let mut keywords = HashMap::new();
        keywords.insert("par", Par);
        keywords.insert("kung", Kung);
        keywords.insert("kungdi", Kungdi);
        keywords.insert("kungwala", Kungwala);
        keywords.insert("ibalik", Ibalik);

        Self {
            source,
            source_iter: source.chars().peekable(),
            start: 0,
            current: 0,
            tokens: Vec::new(),
            bracket_depth: 0,
            keywords,
            diagnostics: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        while self.peek().is_some() {
            self.start = self.current;
            self.lex_token();
        }

        self.add_token(TokenKind::Eof, Some("<EOF>"));
    }

    pub fn transfer_tokens_and_diagnostics(&mut self, module: &mut Module<'src>) {
        let tokens = mem::take(&mut self.tokens);
        let diagnostics = mem::take(&mut self.diagnostics);

        module.set_tokens(tokens);
        module.set_diagnostics(diagnostics);
    }

    fn lex_token(&mut self) {
        // We can safely assume that advancing does not result in a None value here since this is
        // called under the `lex_token` method which checks for Someness beforehand.
        let Some(ch) = self.advance() else {
            unreachable!()
        };

        match ch {
            '(' | '[' => {
                self.bracket_depth += 1;
                match ch {
                    '(' => self.add_token(TokenKind::LParen, None),
                    '[' => self.add_token(TokenKind::LSquare, None),
                    _ => unreachable!(),
                };
            }
            ')' | ']' => {
                self.bracket_depth -= 1;
                match ch {
                    ')' => self.add_token(TokenKind::RParen, None),
                    ']' => self.add_token(TokenKind::RSquare, None),
                    _ => unreachable!(),
                };
            }
            '{' => self.add_token(TokenKind::LBrace, None),
            '}' => self.add_token(TokenKind::RBrace, None),
            ':' => self.add_token(TokenKind::Colon, None),
            ';' => self.add_token(TokenKind::Semicolon, None),
            '<' => {
                if self.match_ch('=') {
                    self.add_token(TokenKind::LesserEq, None);
                } else {
                    self.add_token(TokenKind::Lesser, None);
                }
            }
            '>' => {
                if self.match_ch('=') {
                    self.add_token(TokenKind::GreaterEq, None);
                } else {
                    self.add_token(TokenKind::Greater, None);
                }
            }
            '=' => {
                if self.match_ch('=') {
                    self.add_token(TokenKind::EqualEq, None);
                } else {
                    self.add_token(TokenKind::Equal, None);
                }
            }
            '+' => self.add_token(TokenKind::Plus, None),
            '-' => {
                if self.match_ch('-') {
                    while self.peek().is_some_and(|c| c != '\n') {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Minus, None);
                }
            }
            '*' => self.add_token(TokenKind::Star, None),
            '/' => self.add_token(TokenKind::Slash, None),
            '\n' => {
                // Only emit inferred semicolon when we are not
                // inside `[]` or `()`
                if self.bracket_depth == 0 {
                    self.emit_inferred_semicolon();
                }
            }
            ' ' | '\t' | '\r' => { /* skip irrelevant whitespace */ }
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.lex_identifier(),
            ch if ch.is_ascii_digit() => self.lex_number(),
            _ => {}
        }
    }

    fn lex_identifier(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_ascii_alphanumeric() || ch == '_') {
            self.advance();
        }

        let kind = self
            .keywords
            .get(&self.source[self.span()])
            .cloned()
            .unwrap_or(TokenKind::Identifier);

        self.add_token(kind, None);
    }

    fn lex_number(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit() || ch == '_') {
            self.advance();
        }

        let kind =
            if self.peek() == Some('.') && self.peek_next().is_some_and(|ch| ch.is_ascii_digit()) {
                self.advance();
                while matches!(self.peek(), Some(ch) if ch.is_ascii_digit() || ch == '_') {
                    self.advance();
                }
                TokenKind::FloatLiteral
            } else {
                TokenKind::IntLiteral
            };

        self.add_token(kind, None);
    }

    fn emit_inferred_semicolon(&mut self) {
        if self
            .tokens
            .last()
            .is_some_and(|tok| tok.kind().infers_semicolon())
        {
            self.add_token(TokenKind::Semicolon, Some(";"));
        }
    }

    fn add_token(&mut self, kind: TokenKind, lexeme: Option<&'src str>) {
        let tok = match lexeme {
            Some(s) => Token::new(s, kind, self.span()),
            None => {
                let lexeme = &self.source[self.span()];
                Token::new(lexeme, kind, self.span())
            }
        };

        self.tokens.push(tok)
    }

    fn match_ch(&mut self, ch: char) -> bool {
        if self.peek().is_some_and(|ch2| ch == ch2) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn span(&self) -> Span {
        self.start..self.current
    }

    fn peek(&mut self) -> Option<char> {
        self.source_iter.peek().copied()
    }

    fn peek_next(&mut self) -> Option<char> {
        self.source_iter.clone().nth(1)
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source_iter.next();
        if let Some(ch) = ch {
            self.current += ch.len_utf8();
        };

        ch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<Token<'_>> {
        let mut lexer = Lexer::new(source);
        lexer.run();
        lexer.tokens
    }

    fn kinds(tokens: &[Token<'_>]) -> Vec<TokenKind> {
        tokens.iter().map(|tok| tok.kind().clone()).collect()
    }

    #[test]
    fn lexes_keywords_and_basic_punctuation() {
        let tokens = lex("par kung kungdi kungwala ibalik ( ) [ ] { } : ;");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Par,
                TokenKind::Kung,
                TokenKind::Kungdi,
                TokenKind::Kungwala,
                TokenKind::Ibalik,
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LSquare,
                TokenKind::RSquare,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::Colon,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_identifiers_and_numbers() {
        let tokens = lex("foo _bar baz123 1 42 1_000 12.5 3_000.25");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::IntLiteral,
                TokenKind::IntLiteral,
                TokenKind::IntLiteral,
                TokenKind::FloatLiteral,
                TokenKind::FloatLiteral,
                TokenKind::Eof,
            ]
        );
        assert_eq!(tokens[0].lexeme(), "foo");
        assert_eq!(tokens[3].lexeme(), "1");
        assert_eq!(tokens[6].lexeme(), "12.5");
        assert_eq!(tokens[7].lexeme(), "3_000.25");
    }

    #[test]
    fn infers_semicolons_at_newlines_outside_brackets() {
        let tokens = lex("foo\nbar\n1\n");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Identifier,
                TokenKind::Semicolon,
                TokenKind::Identifier,
                TokenKind::Semicolon,
                TokenKind::IntLiteral,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
        assert_eq!(tokens[1].lexeme(), ";");
        assert_eq!(tokens[1].span(), &(3..4));
    }

    #[test]
    fn does_not_infer_semicolons_inside_parentheses_or_square_brackets() {
        let tokens = lex("(foo\nbar)\n[baz\nqux]\n");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::LParen,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::RParen,
                TokenKind::Semicolon,
                TokenKind::LSquare,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::RSquare,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_line_comments_until_newline() {
        let tokens = lex("foo -- comment\nbar");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Identifier,
                TokenKind::Semicolon,
                TokenKind::Identifier,
                TokenKind::Eof,
            ]
        );
        assert_eq!(tokens[0].lexeme(), "foo");
        assert_eq!(tokens[2].lexeme(), "bar");
    }

    #[test]
    fn lexes_comparison_and_equality_operators() {
        let tokens = lex("< <= > >= = == + - * /");

        assert_eq!(
            kinds(&tokens),
            vec![
                TokenKind::Lesser,
                TokenKind::LesserEq,
                TokenKind::Greater,
                TokenKind::GreaterEq,
                TokenKind::Equal,
                TokenKind::EqualEq,
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Eof,
            ]
        );
    }
}
