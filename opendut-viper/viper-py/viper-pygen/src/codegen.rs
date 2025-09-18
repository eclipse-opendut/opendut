use proc_macro2::Ident;
use crate::args::ParsedPyGenAttributes;
use crate::parse::{Argument, Function, ParsedFn, ParsedInput};

pub fn codegen(_attributes: &ParsedPyGenAttributes, input: &ParsedInput) -> String {

    let functions = input.struct_impl.functions.iter()
        .filter(|function| !function.attributes.skip())
        .filter(|function| !function.parsed_fn.fn_name.to_string().starts_with("__"))
        .map(|function | {

            let args = generate_python_function_arguments(function);

            let Function { attributes, parsed_fn, doc, .. } = function;
            let fn_name = attributes.name.clone().unwrap_or_else(|| function.parsed_fn.fn_name.to_string());
            let fn_return_type = generate_python_return_type(parsed_fn);

            let fn_doc_string = generate_python_doc(doc, 8);

            indoc::formatdoc!("
                def {name}({args}){return_type}:{doc_string}
                        pass\
            ", name = fn_name, args = args, return_type = fn_return_type, doc_string = fn_doc_string)
        }).collect::<Vec<_>>();

    let functions = functions.iter().map(|ident| ident.to_string()).collect::<Vec<_>>().join("\n    "); // TODO: This four spaces may lead to errors due to wrong indention!

    let class_name = input.struct_name.to_string();

    let doc = &input.doc;

    let class_doc_string = generate_python_doc(doc, 4);

    let python = indoc::formatdoc!("
        class {class_name}:{doc_string}
            {functions}
    ", class_name = class_name, doc_string = class_doc_string, functions = functions);
    
    python
}

fn generate_python_function_arguments(function: &Function) -> String {
    let mut required_args = Vec::new();
    let mut optional_args = Vec::new();

    let filtered_args = function.parsed_fn.arguments.iter()
        .filter(|argument| {
            !argument.attributes.skip()
        });

    for argument in filtered_args {
        let Argument { arg_name, attributes, ty } = argument;

        let mut argument = String::new();

        let name = arg_name.to_string();
        if name == "this" || name == "_this" {
            argument.push_str("self")
        } else {
            argument.push_str(&name);
            let argument_type = get_printable_argument_type(ty);
            if let Some(ty) = argument_type {
                argument.push_str(": ");
                argument.push_str(ty);
            }
        }

        if let Some(default_value) = &attributes.default {
            argument.push_str(" = \"");
            argument.push_str(default_value);
            argument.push('"');
            optional_args.push(argument);
        } else {
            required_args.push(argument);
        }
    }

    required_args.extend(optional_args);

    required_args.join(", ")
}

fn generate_python_return_type(parsed_fn: &ParsedFn) -> String {
    let return_type = &parsed_fn.return_type;
    let py_return_type = get_printable_argument_type(return_type);

    let mut fn_return_type = String::new();
    if let Some(py_return_type) = py_return_type {
        fn_return_type.push_str(" -> ");
        fn_return_type.push_str(py_return_type);
    }
    fn_return_type
}

fn generate_python_doc(doc: &[String], indent: usize) -> String {

    let indent = " ".repeat(indent);

    if !doc.is_empty() {
        if doc.len() == 1 {
            let line = doc[0].trim();
            format!("\n{indent}\"\"\"{line}\"\"\"")
        }
        else {
            let doc_string = doc.iter()
                .map(|line| format!("{indent}{}", line.trim()))
                .collect::<Vec<_>>()
                .join("\n");

            format!("\n{indent}\"\"\"\n{doc_string}\n{indent}\"\"\"")
        }
    } else { String::new() }
}

fn get_printable_argument_type(ident: &Option<Ident>) -> Option<&str> {
    ident.as_ref().and_then(|ident| match ident.to_string().as_str() {
        "String" => Some("str"),
        "i8" | "u8" | "i32" | "u32" | "i64" | "u64" | "i128" | "u128" | "isize" | "usize" => Some("int"),
        "f32" | "f64" => Some("float"),
        "bool" => Some("bool"),
        "Vec" => Some("list"),
        "self" | "Self" | "PyResult" | "Result" => None,
        _ => Some("Any")
    })
}
