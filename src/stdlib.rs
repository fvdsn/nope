use crate::penv::{
    FunctionArg,
    Env,
};

#[derive(PartialEq, Debug, Clone)]
pub struct StdlibFunction {
    pub name: String,
    pub args: Vec<FunctionArg>,
}

pub struct Stdlib {
    functions: Vec<StdlibFunction>,
}

impl Stdlib {
    pub fn new() -> Stdlib {
        let mut stdlib = Stdlib {
            functions: vec![],
        };

        let mut def_zero_arg = |name: &str| {
            stdlib.functions.push(StdlibFunction {
                name: name.to_owned(),
                args: vec![],
            });
        };

        def_zero_arg("random");
        def_zero_arg("rand100");
        def_zero_arg("flip-coin");

        let one_arg_func = vec![
            FunctionArg { name: "a".to_owned(), is_func: false, func_arity: 0 },
        ];

        let mut def_one_arg = |name: &str| {
            stdlib.functions.push(StdlibFunction {
                name: name.to_owned(),
                args: one_arg_func.clone(),
            });
        };

        def_one_arg("num");
        def_one_arg("print");
        def_one_arg("echo");
        def_one_arg("neg");
        def_one_arg("return");
        def_one_arg("not");
        def_one_arg("bool");
        def_one_arg("floor");
        def_one_arg("ceil");
        def_one_arg("abs");
        def_one_arg("decr");
        def_one_arg("incr");
        def_one_arg("sin");
        def_one_arg("cos");
        def_one_arg("tan");
        def_one_arg("inv");
        def_one_arg("str");
        def_one_arg("upper");
        def_one_arg("lower");
        def_one_arg("trim");
        def_one_arg("read-text");

        let two_args_func = vec![
            FunctionArg { name: "a".to_owned(), is_func: false, func_arity: 0 },
            FunctionArg { name: "b".to_owned(), is_func: false, func_arity: 0 },
        ];

        let mut def_two_args = |name: &str| {
            stdlib.functions.push(StdlibFunction {
                name: name.to_owned(),
                args: two_args_func.clone(),
            });
        };

        def_two_args("add");
        def_two_args("sub");
        def_two_args("<");
        def_two_args("<=");
        def_two_args(">");
        def_two_args(">=");
        def_two_args("==");
        def_two_args("~=");
        def_two_args("!=");
        def_two_args("!~=");
        def_two_args("max");
        def_two_args("min");
        def_two_args("mult");
        def_two_args("div");
        def_two_args("join-paths");
        def_two_args("write-text");

        let three_args_func = vec![
            FunctionArg { name: "a".to_owned(), is_func: false, func_arity: 0 },
            FunctionArg { name: "b".to_owned(), is_func: false, func_arity: 0 },
            FunctionArg { name: "c".to_owned(), is_func: false, func_arity: 0 },
        ];

        let mut def_three_args = |name: &str| {
            stdlib.functions.push(StdlibFunction {
                name: name.to_owned(),
                args: three_args_func.clone(),
            });
        };

        def_three_args("replace");

        let iterator_args = vec![
            FunctionArg{is_func: false, func_arity:0, name:"array".to_owned()},
            FunctionArg{is_func: true,  func_arity:1, name:"iterator".to_owned()},
        ];

        let mut def_iterator = |name: &str| {
            stdlib.functions.push(StdlibFunction {
                name: name.to_owned(),
                args: iterator_args.clone(),
            });
        };

        def_iterator("iter");

        return stdlib;
    }

    pub fn add_definitions_to_env(&self, env: &mut Env) {
        for func in self.functions.iter() {
            env.push_func_entry(func.name.to_owned(), func.args.clone());
        }
    }
}
