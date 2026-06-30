use std::fmt::{Arguments, Write};

use crate::ast::{
    expr::{Expr, ExprKind},
    stmt::{Stmt, StmtKind},
};

/// A pretty printer for AST nodes
pub struct ASTPrettyPrinter<'ast> {
    input_ast: &'ast [Stmt], // Whole ast tree
    indent: usize,
    indent_size: u8,
    buffer: String,
}

impl<'ast> ASTPrettyPrinter<'ast> {
    pub fn new(input_ast: &'ast [Stmt], indent_size: u8) -> Self {
        Self {
            input_ast,
            indent: 0,
            indent_size,
            buffer: String::new(),
        }
    }

    pub fn run(&mut self) {
        for statement in self.input_ast.iter() {
            self.pretty_statement(statement);
        }

        println!("{}", self.buffer);
    }

    pub fn pretty_statement(&mut self, statement: &Stmt) {
        match statement.kind() {
            StmtKind::NameDeclaration { name, ty, rhs } => {
                self.writeln(format_args!("Name Declaration {{"));
                self.indent();
                self.writeln(format_args!("name: {}", name.lexeme()));
                self.writeln(format_args!("ty: {:?}", ty));
                let rhs = self.pretty_expression(rhs);
                self.writeln(format_args!("rhs: {}", rhs));
                self.dedent();
                self.writeln(format_args!("}}"));
            }
            StmtKind::Expression { expr } => {
                let expr = self.pretty_expression(expr);
                self.writeln(format_args!("Expression Statement ({})", expr));
            }
        }
    }

    pub fn pretty_expression(&mut self, expression: &Expr) -> String {
        let mut expr_buf = String::new();
        match expression.kind() {
            ExprKind::Integer(token) => {
                write!(expr_buf, "Integer ({})", token.lexeme());
            }
            ExprKind::Float(token) => {
                write!(expr_buf, "Float ({})", token.lexeme());
            }
            ExprKind::Identifier(token) => {
                write!(expr_buf, "Identifier ({})", token.lexeme());
            }
            ExprKind::Binary { lhs, rhs, op } => {
                writeln!(&mut expr_buf, "Binary {{");
                self.indent();
                let lhs = self.pretty_expression(lhs);
                self.writeln_buf(&mut expr_buf, format_args!("lhs: {}", lhs));
                let rhs = self.pretty_expression(rhs);
                self.writeln_buf(&mut expr_buf, format_args!("rhs: {}", rhs));
                self.writeln_buf(&mut expr_buf, format_args!("op: {}", op.lexeme()));
                self.dedent();
                write!(&mut expr_buf, "{}}}", " ".repeat(self.indent));
            }
        };

        expr_buf
    }

    pub fn indent(&mut self) {
        self.indent += self.indent_size as usize;
    }

    pub fn dedent(&mut self) {
        self.indent -= self.indent_size as usize;
    }

    pub fn writeln(&mut self, fmt: Arguments) {
        write!(self.buffer, "{}", " ".repeat(self.indent));
        writeln!(self.buffer, "{}", fmt);
    }

    pub fn writeln_buf(&mut self, buf: &mut impl Write, fmt: Arguments) {
        write!(buf, "{}", " ".repeat(self.indent));
        writeln!(buf, "{}", fmt);
    }
}
