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
    pub is_global: bool,
    pub is_const: bool,
    pub func_args: Vec<FunctionArg>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Env {
    entries: Vec<EnvEntry>,
}

impl Env {
    pub fn new() -> Env {
        return Env {
            entries:vec![],
        };
    }

    pub fn print(&self) {
        println!("Env:");
        for entry in self.entries.iter() {
            println!("  {}{}", entry.name, if entry.is_func { format!("|{}|", entry.func_args.len()) } else { "".to_string() });
        }
    }

    pub fn push_value_entry(&mut self, name: String, is_global: bool, is_const: bool) {
        self.entries.push(EnvEntry {
            name,
            is_global,
            is_const,
            is_func:false,
            func_args:vec![],
        });
    }

    pub fn push_func_entry(
        &mut self,
        name: String,
        is_global: bool,
        is_const: bool,
        args: Vec<FunctionArg>,
    ) {
        if name == "_" {    // _ must keep having the void value
            self.entries.push(EnvEntry {
                name,
                is_global: true,
                is_const: true,
                is_func: false,
                func_args:vec![],
            });
        } else {
            self.entries.push(
                EnvEntry {
                    name,
                    is_global,
                    is_const,
                    is_func:true,
                    func_args:args,
                });
        }
    }

    pub fn push_arg_func_entry(&mut self, name: String, is_global: bool, is_const: bool, argc:usize) {
        let mut func_args: Vec<FunctionArg> = vec![];
        for i in 0..argc {
            func_args.push(FunctionArg {
                name: format!("arg{}",i+1),
                is_func: false,
                func_arity: 0,
            });
        }
        self.entries.push(EnvEntry {
            name,
            is_global,
            is_const,
            is_func:true,
            func_args,
        });
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

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.entries.len()
    }
}

