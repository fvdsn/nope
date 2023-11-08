use std::collections::HashMap;

use crate::penv::{
    FunctionArg,
    Env,
};
use crate::chunk::Instruction;

#[derive(PartialEq, Debug, Clone)]
pub struct StdlibFunction {
    pub name: String,
    pub args: Vec<FunctionArg>,
    pub instructions: Vec<Instruction>,
}

pub struct Stdlib {
    functions: Vec<StdlibFunction>,
    functions_map: HashMap<String, StdlibFunction>,
}

impl Stdlib {
    pub fn new() -> Stdlib {
        let mut stdlib = Stdlib {
            functions: vec![],
            functions_map: HashMap::new(),
        };

        let mut def_zero_arg = |name: &str, instructions: Vec<Instruction>| {
            stdlib.functions.push(StdlibFunction {
                instructions,
                name: name.to_owned(),
                args: vec![],
            });
        };

        def_zero_arg("random", vec![Instruction::Random]);
        def_zero_arg("rand100", vec![
            Instruction::Random,
            Instruction::ConstantNum(100.0),
            Instruction::Multiply,
            Instruction::Floor,
        ]);
        def_zero_arg("flip_coin", vec![
            Instruction::Random,
            Instruction::ConstantNum(0.5),
            Instruction::GreaterOrEqual,
        ]);
        for num in [4, 6, 8, 10, 12, 20, 100] {
            def_zero_arg(&format!("d{}", num), vec![
                Instruction::Random,
                Instruction::ConstantNum(f64::from(num)),
                Instruction::Multiply,
                Instruction::Ceil,
            ]);
        }

        let one_arg_func = vec![
            FunctionArg { name: "a".to_owned(), is_func: false, func_arity: 0 },
        ];

        let mut def_one_arg = |name: &str, instructions: Vec<Instruction>| {
            stdlib.functions.push(StdlibFunction {
                instructions,
                name: name.to_owned(),
                args: one_arg_func.clone(),
            });
        };

        def_one_arg("num",    vec![Instruction::Num]);
        def_one_arg("print",  vec![Instruction::Print]);
        def_one_arg("echo",   vec![Instruction::Echo]);
        def_one_arg("neg",    vec![Instruction::Negate]);
        def_one_arg("return", vec![Instruction::Return,]);
        def_one_arg("not",    vec![Instruction::Not]);
        def_one_arg("bool",   vec![Instruction::Bool]);
        def_one_arg("floor",  vec![Instruction::Floor]);
        def_one_arg("ceil",   vec![Instruction::Ceil]);
        def_one_arg("abs",    vec![Instruction::Abs]);
        def_one_arg("acos",   vec![Instruction::Acos]);
        def_one_arg("decr",   vec![Instruction::Decr]);
        def_one_arg("incr",   vec![Instruction::Incr]);
        def_one_arg("sin",    vec![Instruction::Sin]);
        def_one_arg("cos",    vec![Instruction::Cos]);
        def_one_arg("tan",    vec![Instruction::Tan]);
        def_one_arg("inv",    vec![Instruction::Inv]);
        def_one_arg("str",    vec![Instruction::Str]);
        def_one_arg("upper",  vec![Instruction::Upper]);
        def_one_arg("lower",  vec![Instruction::Lower]);
        def_one_arg("trim",   vec![Instruction::Trim]);
        def_one_arg("shh",    vec![Instruction::Silence]);
        def_one_arg("read_text", vec![Instruction::ReadTextFileSync]);
        def_one_arg("is_even", vec![
            Instruction::ConstantNum(2.0),
            Instruction::Modulo,
            Instruction::ConstantNum(0.0),
            Instruction::Equal,
        ]);
        def_one_arg("is_odd", vec![
            Instruction::ConstantNum(2.0),
            Instruction::Modulo,
            Instruction::ConstantNum(0.0),
            Instruction::Equal,
            Instruction::Not,
        ]);


        let two_args_func = vec![
            FunctionArg { name: "a".to_owned(), is_func: false, func_arity: 0 },
            FunctionArg { name: "b".to_owned(), is_func: false, func_arity: 0 },
        ];

        let mut def_two_args = |name: &str, instructions: Vec<Instruction>| {
            stdlib.functions.push(StdlibFunction {
                instructions,
                name: name.to_owned(),
                args: two_args_func.clone(),
            });
        };

        def_two_args("add",  vec![Instruction::Add]);
        def_two_args("sub",  vec![Instruction::Subtract]);
        def_two_args("le",    vec![Instruction::Less]);
        def_two_args("leq",   vec![Instruction::LessOrEqual]);
        def_two_args("ge",    vec![Instruction::Greater]);
        def_two_args("geq",   vec![Instruction::GreaterOrEqual]);
        def_two_args("eq",   vec![Instruction::Equal]);
        def_two_args("aeq",   vec![Instruction::AlmostEqual]);
        def_two_args("neq",   vec![Instruction::Equal, Instruction::Not]);
        def_two_args("naeq",  vec![Instruction::AlmostEqual, Instruction::Not]);
        def_two_args("max",  vec![Instruction::Max]);
        def_two_args("min",  vec![Instruction::Min]);
        def_two_args("mult", vec![Instruction::Multiply]);
        def_two_args("div",  vec![Instruction::Divide]);
        def_two_args("modulo",     vec![Instruction::Modulo]);
        def_two_args("join_paths", vec![Instruction::JoinPaths]);
        def_two_args("write_text", vec![Instruction::WriteTextFileSync]);

        let three_args_func = vec![
            FunctionArg { name: "a".to_owned(), is_func: false, func_arity: 0 },
            FunctionArg { name: "b".to_owned(), is_func: false, func_arity: 0 },
            FunctionArg { name: "c".to_owned(), is_func: false, func_arity: 0 },
        ];

        let mut def_three_args = |name: &str, instruction: Instruction| {
            stdlib.functions.push(StdlibFunction {
                instructions: vec![instruction],
                name: name.to_owned(),
                args: three_args_func.clone(),
            });
        };

        def_three_args("replace", Instruction::Replace);

        let iterator_args = vec![
            FunctionArg{is_func: false, func_arity:0, name:"array".to_owned()},
            FunctionArg{is_func: true,  func_arity:1, name:"iterator".to_owned()},
        ];

        let mut def_iterator = |name: &str, instructions: Vec<Instruction>| {
            stdlib.functions.push(StdlibFunction {
                instructions,
                name: name.to_owned(),
                args: iterator_args.clone(),
            });
        };

        def_iterator("iter", vec![]); // not implemented, used in parsing tests

        for function in stdlib.functions.iter() {
            stdlib.functions_map.insert(function.name.to_owned(), function.clone());
        }

        return stdlib;
    }

    pub fn add_definitions_to_env(&self, env: &mut Env) {
        for func in self.functions.iter() {
            env.push_func_entry(func.name.to_owned(), func.args.clone());
        }
    }

    pub fn get_function_instructions(&self, name: &str) -> Option<&Vec<Instruction>> {
        if let Some(function) = self.functions_map.get(name) {
            return Some(&function.instructions);
        } else {
            return None;
        }
    }

    pub fn make_env(&self) -> Env {
        let mut env = Env::new();
        self.add_definitions_to_env(&mut env);
        return env;
    }
}
