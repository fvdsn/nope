

use rand::seq::SliceRandom;
use std::rc::Rc;
use std::cell::RefCell;

//use rustyline::error::ReadlineError;
//use rustyline::{DefaultEditor};
//use rustyline::validate::{ValidationContext, ValidationResult, Validator};
//use rustyline::{Completer, Helper, Highlighter, Hinter};
//use rustyline::{Editor, Result};
//
//use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use rustyline::validate::{Validator, ValidationResult, ValidationContext};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter };

use crate::{
    penv::Env,
    parser::Parser,
    stdlib::Stdlib,
    vm::Vm,
    config::NopeConfig,
};

use colored::*;

fn print_colored_line(len: usize, c:&str) {
    print!("  ");
    for _ in 0..len {
        print!("{}", c.blue());
    }
    println!();
}

fn print_banner() {
    let messages: Vec<&str> = vec![
        "Enjoy!",
        "You can do it!",
        "Have fun!",
        "You are amazing!",
        "Turbo mode activated!",
        "All Systems are GO!",
        "Today is a good day!",
        "Peace ✌️",
        "Make something incredible!",
        "Make something small!",
        "Make something fun!",
        "Make something cute!",
        "Make something cool!",
    ];
    let mut rng = rand::thread_rng();
    let banner = format!(
        "Welcome to the NOPE repl! {}",
        messages.choose(&mut rng).expect("should not happen")
    );
    println!();
    print_colored_line(banner.chars().count()+4, "-");
    println!("  {} {} {}", ":".blue(), banner, ":".blue());
    print_colored_line(banner.chars().count()+4, "=");
    println!();
}

#[derive(PartialEq, Debug, Clone)]
struct SharedEnv {
    env: Env,
}

#[derive(Completer, Highlighter, Helper, Hinter)]
struct InputValidator {
    shared_env: Rc<RefCell<SharedEnv>>,
}

impl Validator for InputValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        use ValidationResult::{Incomplete, Valid};
        let input = ctx.input();
        let config = NopeConfig{ debug:false, trace: false, echo_result:false };
        let shared = (*self.shared_env).clone();
        let mut parser = Parser::new_with_env(
            config,
            shared.into_inner().env.clone(),
            input.to_string()
        );
        parser.parse();

        let result = if parser.incomplete() {
            Incomplete
        } else {
            Valid(None)
        };

        return Ok(result);
    }
}


pub fn repl(vm: &mut Vm) {
    let mut rl = Editor::new().expect("could not activate line editor");
    let stdlib = Stdlib::new();
    let env = stdlib.make_env();
    let shared_env = Rc::new(RefCell::new(SharedEnv {env}));
    let h = InputValidator {shared_env: Rc::clone(&shared_env)};
    rl.set_helper(Some(h));

    print_banner();
    loop {
        let readline = rl.readline(&format!("{}", "> ".blue()));
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).ok();
                vm.interpret(line);
                if let Some(env) = vm.get_copy_of_last_env() {
                    shared_env.replace(SharedEnv {env: env.clone()});
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("  {}", "exit (^C)".blue());
                break
            },
            Err(ReadlineError::Eof) => {
                println!("  {}", "exit (^D)".blue());
                break
            },
            Err(err) => {
                println!("  {}", format!("Error: {:?}", err).red());
                break
            }
        }
    }
}
