#![allow(unused)]

use std::{fs, path::PathBuf};

use crate::{lexer::Lexer, module::Module, parser::Parser, sema::SemanticAnalyzer};

mod ast;
mod diagnostic;
mod lexer;
mod module;
mod parser;
mod prelude;
mod sema;
mod symbol;
mod token;
mod toltype;

fn main() {
    let source_code = {
        use clap::Parser;

        let args = Args::parse();
        fs::read_to_string(args.input).unwrap()
    };

    let mut modul = Module::new(source_code);

    let source_code2 = modul.source_code_arc();
    let mut lexer = Lexer::new(&source_code2);
    let tokens = lexer.run();

    for token in tokens.iter() {
        println!(
            "{} => {:?} at [{}:{}]",
            token.lexeme(),
            token.kind(),
            token.span().start,
            token.span().end
        )
    }

    lexer.transfer_diagnostics(&mut modul);

    let mut parser = Parser::new(tokens, &mut modul);
    parser.run();
    modul.display_ast();

    let mut semantic_analyzer = SemanticAnalyzer::new(&mut modul);
    semantic_analyzer.run();

    modul.report_diagnostics();
}

#[derive(clap::Parser)]
struct Args {
    /// Path to the input file
    #[arg(help("Path na nagtuturo sa input file"))]
    input: PathBuf,
}
