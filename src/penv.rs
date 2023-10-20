
#[derive(PartialEq, Debug, Clone)]
pub struct FunctionArg {
    pub name: String,
    pub is_func: bool,
    pub func_arity: usize,
}

#[derive(PartialEq, Debug, Clone)]
pub struct EnvEntry {
    pub name: String,
    pub is_func: bool,
    pub func_args: Vec<FunctionArg>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Env {
    entries: Vec<EnvEntry>,
    checkpoint: usize,
}

impl Env {
    pub fn new() -> Env {
        return Env {
            entries:vec![],
            checkpoint: 0,
        };
    }

    pub fn new_with_stdlib() -> Env {
        let mut env = Env::new();
        env.set_stdlib();
        return env;
    }

    pub fn print(&self) {
        println!("Env:");
        for entry in self.entries.iter() {
            println!("  {}{}", entry.name, if entry.is_func { format!("|{}|", entry.func_args.len()) } else { "".to_string() });
        }
    }

    pub fn push_value_entry(&mut self, name: String) {
        self.entries.push(EnvEntry { name, is_func:false, func_args:vec![] });
    }

    pub fn push_func_entry(&mut self, name: String, args: Vec<FunctionArg>) {
        if name == "_" {    // _ must keep having the void value
            self.entries.push(EnvEntry { name, is_func:false, func_args:vec![] });
        } else {
            self.entries.push(EnvEntry { name, is_func:true, func_args:args });
        }
    }

    pub fn push_arg_func_entry(&mut self, name: String, argc:usize) {
        let mut func_args: Vec<FunctionArg> = vec![];
        for i in 0..argc {
            func_args.push(FunctionArg {
                name: format!("arg{}",i+1),
                is_func: false,
                func_arity: 0,
            });
        }
        self.entries.push(EnvEntry { name, is_func:true, func_args });
    }

    pub fn pop_entry(&mut self) {
        self.entries.pop();
    }

    pub fn get_entry(&self, name: &String) -> Option<EnvEntry> {
        if self.entries.is_empty() {
            return None;
        }
        let mut i = self.entries.len() - 1;
        loop {
            let entry = &self.entries[i];
            if entry.name == *name {
                let _entry: EnvEntry = entry.clone();
                return Some(_entry);
            }
            if i > 0 {
                i -= 1;
            } else {
                break;
            }
        }
        return None;
    }

    fn set_stdlib(&mut self) {
        // |a f:1|
        self.entries.push(
            EnvEntry{ name: "iter".to_owned(), is_func: true, 
                func_args: vec![
                    FunctionArg{is_func: false, func_arity:0, name:"array".to_owned()},
                    FunctionArg{is_func: true,  func_arity:1, name:"iterator".to_owned()},
                ]
            }
        );

        // |f:2 a|
        for name in ["map", "filter"] {
            self.entries.push(
                EnvEntry{ name: name.to_owned(), is_func: true, 
                    func_args: vec![
                        FunctionArg{is_func: true,  func_arity:2, name:"iterator".to_owned()},
                        FunctionArg{is_func: false, func_arity:0, name:"array".to_owned()},
                    ]
                }
            );
        }
        
        // ||
        for name in ["random", "rand100", "flip-coin"] {
            self.entries.push(
                EnvEntry{ name: name.to_owned(), is_func: true, func_args: vec![]}
            );
        }

        // |a|
        for name in [
            "range", "increment",
            "/max", "/min", "/and",
            "/or", "/eq", "/add", "/mult", "len",
            // implemented
            "num", "print", "echo", "neg", "return", "not", "bool",
            "floor", "ceil", "abs", "decr", "incr", "sin", "cos", 
            "tan", "inv", "str", "upper", "lower", "trim",
            "read-text",        
        ] {
            self.entries.push(
                EnvEntry{ name: name.to_owned(), is_func: true, 
                    func_args: vec![
                        FunctionArg{is_func: false, func_arity:0, name:"arg".to_owned()},
                    ]
                }
            );
        }
        // |a b|
        for name in vec![
            "and", "or", "eq", "neq", "mod",
            "exp",
            //implemented
            "add", "sub",
            "<", "<=", ">", ">=", "==", "~=", "!=", "!~=",
            "max", "min", "mult", "div", "join-paths", "write-text"
        ] {
            self.entries.push(
                EnvEntry{ name: name.to_owned(), is_func: true, 
                    func_args: vec![
                        FunctionArg{is_func: false, func_arity:0, name:"arg1".to_owned()},
                        FunctionArg{is_func: false, func_arity:0, name:"arg2".to_owned()},
                    ]
                }
            );
        }
        // |a b c|
        for name in vec![
            //implemented
            "replace",
        ] {
            self.entries.push(
                EnvEntry{ name: name.to_owned(), is_func: true, 
                    func_args: vec![
                        FunctionArg{is_func: false, func_arity:0, name:"arg1".to_owned()},
                        FunctionArg{is_func: false, func_arity:0, name:"arg2".to_owned()},
                        FunctionArg{is_func: false, func_arity:0, name:"arg3".to_owned()},
                    ]
                }
            );
        }
    }
}

