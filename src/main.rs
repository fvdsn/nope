#![allow(clippy::needless_return)]

use std::fs;
use clap::{Arg, Command};

mod config;
mod tokenizer;
mod parser;
mod units;
mod chunk;
mod vm;
mod repl;
mod gc;
mod objects;


use crate::{
    tokenizer::Tokenizer,
    parser::Parser,
    vm::Vm,
    config::NopeConfig,
    repl::repl,
};


fn main() {

    let m = Command::new("nope")
        .version("0.1.0")
        .about("The nope interpreter")
        .long_about("
            interpreter for the nope programming languages. very early stages.
        ")
        .author("Frédéric van der Essen")
        .arg(
            Arg::new("tokenize")
                .long("tokenize")
                .short('t')
                .takes_value(false)
                .help("Perform token validation of the source code")
                .required(false)
        )
        .arg(
            Arg::new("parse")
                .long("parse")
                .short('p')
                .takes_value(false)
                .help("Parses and validates the source code")
                .required(false)
        )
        .arg(
            Arg::new("ast")
                .long("ast")
                .short('a')
                .takes_value(false)
                .help("Prints the ast of the program")
                .required(false)
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .takes_value(false)
                .help("Activate debug logs")
                .required(false)
        )
        .arg(
            Arg::new("eval")
                .long("eval")
                .short('e')
                .takes_value(true)
                .help("Evaluates the code provided as argument value")
                .required(false)
        )
        .arg(
            Arg::new("filename")
                .help("The path to the source code")
                .index(1)
                .required(false)
        )
        .after_help("")
        .get_matches();

    let mut config = NopeConfig {
        debug: m.is_present("debug"),
        echo_result: false,
    };

    if !m.is_present("eval") || !m.is_present("filename") {
        config.echo_result = true;
        let mut vm = Vm::new(config);
        repl(&mut vm);
        return;
    }

    let source = if m.is_present("eval") {
        String::from(m.value_of("eval").expect("no code provided to --eval argument"))
    } else {
        let filename = m.value_of("filename").expect("No file argument provided");
        fs::read_to_string(filename).expect("Could not read file")
    };

    if m.is_present("tokenize") {
        let mut tokenizer = Tokenizer::new(source);
        tokenizer.tokenize();
        tokenizer.print();
    } else if m.is_present("parse") {
        let mut parser = Parser::new(config, source);
        parser.parse();
        parser.tokenizer.print();
        parser.print();
    } else if m.is_present("ast") {
        let mut parser = Parser::new(config, source);
        parser.parse();
        parser.pretty_print();
    } else {
        let mut vm = Vm::new(config);
        vm.interpret(source);
    }
}
