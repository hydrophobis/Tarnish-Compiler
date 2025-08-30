mod tokenizer;
use std::{fmt::format, vec, collections::HashMap};

use tokenizer::{tokenize, Token};

use crate::tokenizer::detokenize;

pub static DEBUG: bool = false;

// AST
#[derive(Debug)]
pub enum Node {
    Var { name: String, value: String },
    Print { expr: String },
}

#[derive(Debug)]
pub struct Class {
    name: String,
    namespace: Option<String>,
    variables: Vec<Variable>,
    functions: Vec<Function>,
    operators: Vec<OperatorOverload>,
}

impl ToString for Class {
    fn to_string(&self) -> String {
        let full_name = match &self.namespace {
            Some(ns) => format!("{}_{}", ns, self.name),
            None => self.name.clone(),
        };

        let mut s = format!("typedef struct {{ ");

        for var in &self.variables {
            s.push_str(var.to_string().as_str());
        }

        s.push_str(format!(" }} {};\n", &full_name).as_str());

        for func in &self.functions {
            s.push_str(func.to_string().as_str());
        }

        for op in &self.operators {
            s.push_str(op.to_string().as_str());
        }
        s
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
    type_: String
}

impl ToString for Variable {
    fn to_string(&self) -> String {
        format!("{} {};", self.type_, self.name)
    }
}

#[derive(Debug)]
struct Function {
    class_name: String,
    namespace: Option<String>,
    name: String,
    return_type: String,
    params: Vec<String>,
    body_tokens: Vec<Token>,
}

impl ToString for Function {
    fn to_string(&self) -> String {
        let token_strings: Vec<String> = self.body_tokens.iter().map(|t| {
            match t {
                Token::Identifier(s)
                | Token::Number(s)
                | Token::StringLit(s)
                | Token::CharLit(s)
                | Token::Symbol(s)
                | Token::Comment(s) => s.clone(),
                Token::Newline => "\n".to_string(),
                Token::Eof => "".to_string(),
            }
        }).collect();

        let joined = token_strings.join(" ");
        let full_class_name = match &self.namespace {
            Some(ns) => format!("{}_{}", ns, self.class_name),
            None => self.class_name.clone(),
        };
        
        let params = if self.params.is_empty() {
            String::new()
        } else {
            ",".to_owned() + &self.params.join(", ")
        };

        format!(
            "{} {}_{}({} self{}){{{}}}",
            self.return_type,
            full_class_name,
            self.name,
            full_class_name,
            params,
            joined
        )
    }
}

#[derive(Debug)]
struct OperatorOverload {
    class_name: String,
    namespace: Option<String>,
    operator: String,
    return_type: String,
    params: Vec<String>,
    body_tokens: Vec<Token>,
}

impl ToString for OperatorOverload {
    fn to_string(&self) -> String {
        let token_strings: Vec<String> = self.body_tokens.iter().map(|t| {
            match t {
                Token::Identifier(s)
                | Token::Number(s)
                | Token::StringLit(s)
                | Token::CharLit(s)
                | Token::Symbol(s)
                | Token::Comment(s) => s.clone(),
                Token::Newline => "\n".to_string(),
                Token::Eof => "".to_string(),
            }
        }).collect();

        let joined = token_strings.join(" ");
        let full_class_name = match &self.namespace {
            Some(ns) => format!("{}_{}", ns, self.class_name),
            None => self.class_name.clone(),
        };
        
        let operator_name = match self.operator.as_str() {
            "+" => "add",
            "-" => "sub",
            "*" => "mul",
            "/" => "div",
            "==" => "eq",
            "!=" => "neq",
            "<" => "lt",
            ">" => "gt",
            "<=" => "le",
            ">=" => "ge",
            "+=" => "add_assign",
            "-=" => "sub_assign",
            "*=" => "mul_assign",
            "/=" => "div_assign",
            "++" => "increment",
            "--" => "decrement",
            "[]" => "index",
            _ => "unknown_op",
        };
        
        format!("{} {}_operator_{}({} self, {}){{{}}}", 
                self.return_type, full_class_name, operator_name, 
                full_class_name, self.params.join(", "), joined)
    }
}

#[derive(Debug, Clone)]
struct Namespace {
    name: String,
    classes: Vec<String>,
    functions: Vec<String>,
}

fn parse_namespace_declaration(tokens: &[Token], start_index: usize) -> Option<(String, usize)> {
    if DEBUG {println!("DEBUG: Checking for namespace at token {}", start_index);}
    
    if let Token::Identifier(keyword) = &tokens[start_index] {
        if keyword == "namespace" {
            if let Some(Token::Identifier(namespace_name)) = tokens.get(start_index + 1) {
                if let Some(Token::Symbol(brace)) = tokens.get(start_index + 2) {
                    if brace == "{" {
                        if DEBUG {println!("DEBUG: Found namespace: {}", namespace_name);}
                        return Some((namespace_name.clone(), start_index + 3));
                    }
                }
            }
        }
    }
    None
}

fn find_namespace_end(tokens: &[Token], start_index: usize) -> usize {
    let mut brace_level = 1;
    let mut i = start_index;
    
    while i < tokens.len() && brace_level > 0 {
        match &tokens[i] {
            Token::Symbol(s) if s == "{" => brace_level += 1,
            Token::Symbol(s) if s == "}" => brace_level -= 1,
            _ => {}
        }
        i += 1;
    }
    i
}

fn parse_operator_overload(tokens: &[Token], start_index: usize, class_name: String, namespace: Option<String>) -> Option<(OperatorOverload, usize)> {
    if DEBUG {println!("DEBUG: Checking for operator overload at token {}", start_index);}
    
    // Look for: return_type "operator" operator_symbol "(" params ")" "{" body "}"
    if start_index + 4 >= tokens.len() {
        return None;
    }
    
    if let Token::Identifier(return_type) = &tokens[start_index] {
        if let Token::Identifier(keyword) = &tokens[start_index + 1] {
            if keyword == "operator" {
                if let Token::Symbol(op_symbol) = &tokens[start_index + 2] {
                    if let Token::Symbol(left_paren) = &tokens[start_index + 3] {
                        if left_paren == "(" {
                            if DEBUG {println!("DEBUG: Found operator overload: {} operator{}", return_type, op_symbol);}
                            
                            // Parse parameters
                            let mut params = Vec::new();
                            let mut p = start_index + 4;
                            
                            // Parse parameters until )
                            while p < tokens.len() {
                                if let Token::Symbol(sym) = &tokens[p] {
                                    if sym == ")" {
                                        p += 1; // move past )
                                        break;
                                    }
                                    if sym == "," {
                                        p += 1; // skip comma
                                        continue;
                                    }
                                }
                                
                                // Try to parse type name pair
                                if let Token::Identifier(param_type) = &tokens[p] {
                                    if p + 1 < tokens.len() {
                                        if let Token::Identifier(param_name) = &tokens[p + 1] {
                                            let param = format!("{} {}", param_type, param_name);
                                            if DEBUG {println!("DEBUG: Found operator parameter: {}", param);}
                                            params.push(param);
                                            p += 2;
                                            continue;
                                        }
                                    }
                                }
                                p += 1;
                            }
                            
                            // Find opening brace
                            while p < tokens.len() {
                                if let Token::Symbol(s) = &tokens[p] {
                                    if s == "{" {
                                        break;
                                    }
                                }
                                p += 1;
                            }
                            
                            // Parse body
                            let mut body_tokens = Vec::new();
                            if p < tokens.len() {
                                if let Token::Symbol(s) = &tokens[p] {
                                    if s == "{" {
                                        let mut brace_level = 1;
                                        let mut b = p + 1;
                                        while b < tokens.len() && brace_level > 0 {
                                            match &tokens[b] {
                                                Token::Symbol(s) if s == "{" => brace_level += 1,
                                                Token::Symbol(s) if s == "}" => brace_level -= 1,
                                                _ => {},
                                            }
                                            if brace_level > 0 {
                                                body_tokens.push(tokens[b].clone());
                                            }
                                            b += 1;
                                        }
                                        
                                        let operator_overload = OperatorOverload {
                                            class_name: class_name.clone(),
                                            namespace: namespace.clone(),
                                            operator: op_symbol.clone(),
                                            return_type: return_type.clone(),
                                            params,
                                            body_tokens,
                                        };
                                        
                                        return Some((operator_overload, b));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

fn parse_functions_with_operators(tokens: &[Token], class: String, namespace: Option<String>) -> (Vec<Function>, Vec<OperatorOverload>) {
    if DEBUG {println!("DEBUG: Starting parse_functions_with_operators with {} tokens", tokens.len());}
    let mut functions = Vec::new();
    let mut operators = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if DEBUG && i % 50 == 0 {println!("DEBUG: parse_functions_with_operators - checking token {} of {}", i, tokens.len());}
        
        // Try to parse operator overload first
        if let Some((op_overload, next_i)) = parse_operator_overload(tokens, i, class.clone(), namespace.clone()) {
            operators.push(op_overload);
            i = next_i;
            continue;
        }
        
        // Then try to parse regular function
        if i + 2 < tokens.len() {
            // look for return_type identifier (
            if let Token::Identifier(ret_type) = &tokens[i] {
                if let Token::Identifier(name) = &tokens[i + 1] {
                    if let Token::Symbol(sym) = &tokens[i + 2] {
                        if sym == "(" {
                            if DEBUG {println!("DEBUG: Found function: {} {}", ret_type, name);}
                            
                            // parse params until )
                            let mut params = Vec::new();
                            let mut p = i + 3;
                            
                            // Parse parameters
                            while p < tokens.len() {
                                if let Token::Symbol(sym) = &tokens[p] {
                                    if sym == ")" {
                                        p += 1;
                                        break;
                                    }
                                    if sym == "," {
                                        p += 1;
                                        continue;
                                    }
                                }
                                
                                if let Token::Identifier(param_type) = &tokens[p] {
                                    if p + 1 < tokens.len() {
                                        if let Token::Identifier(param_name) = &tokens[p + 1] {
                                            let param = format!("{} {}", param_type, param_name);
                                            params.push(param);
                                            p += 2;
                                            continue;
                                        }
                                    }
                                }
                                p += 1;
                            }

                            // Find opening brace
                            while p < tokens.len() {
                                if let Token::Symbol(s) = &tokens[p] {
                                    if s == "{" {
                                        break;
                                    }
                                }
                                p += 1;
                            }

                            // Parse body
                            let mut body_tokens = Vec::new();
                            if p < tokens.len() {
                                if let Token::Symbol(s) = &tokens[p] {
                                    if s == "{" {
                                        let mut brace_level = 1;
                                        let mut b = p + 1;
                                        while b < tokens.len() && brace_level > 0 {
                                            match &tokens[b] {
                                                Token::Symbol(s) if s == "{" => brace_level += 1,
                                                Token::Symbol(s) if s == "}" => brace_level -= 1,
                                                _ => {},
                                            }
                                            if brace_level > 0 {
                                                body_tokens.push(tokens[b].clone());
                                            }
                                            b += 1;
                                        }
                                        i = b;
                                    } else {
                                        i += 1;
                                        continue;
                                    }
                                }
                            } else {
                                break;
                            }

                            functions.push(Function {
                                class_name: class.clone(),
                                namespace: namespace.clone(),
                                name: name.clone(),
                                return_type: ret_type.clone(),
                                params,
                                body_tokens,
                            });
                            continue;
                        }
                    }
                }
            }
        }
        i += 1;
    }

    if DEBUG {println!("DEBUG: parse_functions_with_operators completed, found {} functions and {} operators", functions.len(), operators.len());}
    (functions, operators)
}

fn collect_all_variables_with_namespace(tokens: &[Token], class_names: &HashMap<String, String>) -> Vec<Variable> {
    if DEBUG {println!("DEBUG: Collecting all variables from {} tokens with namespace support", tokens.len());}
    let mut variables = Vec::new();
    let mut i = 0;

    while i + 2 < tokens.len() {
        if let Token::Identifier(type_) = &tokens[i] {
            if let Token::Identifier(name) = &tokens[i + 1] {
                if let Token::Symbol(sym) = &tokens[i + 2] {
                    if sym == ";" {
                        // Vector e;
                        if DEBUG {
                            println!("DEBUG: Found variable: {} {}", type_, name);
                        }
                        variables.push(Variable {
                            name: name.clone(),
                            type_: type_.clone(),
                        });
                        i += 3;
                        continue;
                    } else if sym == "=" {
                        // Vector e = ...;
                        if DEBUG {
                            println!(
                                "DEBUG: Found variable with assignment: {} {}",
                                type_, name
                            );
                        }
                        variables.push(Variable {
                            name: name.clone(),
                            type_: type_.clone(),
                        });

                        // Skip to the semicolon after the assignment expression
                        let mut j = i + 3;
                        while j < tokens.len() {
                            if let Token::Symbol(s) = &tokens[j] {
                                if s == ";" {
                                    break;
                                }
                            }
                            j += 1;
                        }
                        i = j + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }


    if DEBUG {println!("DEBUG: Found {} variables total", variables.len());}
    variables
}

fn parse_function_calls_with_operators(tokens: Vec<Token>, class_names: HashMap<String, String>) -> Vec<Token> {
    if DEBUG {println!("DEBUG: Starting parse_function_calls_with_operators with {} tokens and {} classes", tokens.len(), class_names.len());}
    
    let variables = collect_all_variables_with_namespace(&tokens, &class_names);
    let mut out_tokens: Vec<Token> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if i % 200 == 0 {
            if DEBUG {println!("DEBUG: parse_function_calls_with_operators - processing token {} of {}", i, tokens.len());}
        }

        // Handle operator overloading
        if let Token::Identifier(left_operand) = &tokens[i] {
            if let Some(var) = variables.iter().find(|v| &v.name == left_operand) {
                // Check for binary operators: obj + other, obj == other, etc.
                if i + 2 < tokens.len() {
                    if let Token::Symbol(operator) = &tokens[i + 1] {
                        if matches!(operator.as_str(), "+" | "-" | "*" | "/" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "+=" | "-=" | "*=" | "/=") {
                            if DEBUG {println!("DEBUG: Found binary operator: {} {} ...", left_operand, operator);}
                            
                            let class_with_namespace = class_names.get(&var.type_).unwrap_or(&var.type_);
                            let operator_name = match operator.as_str() {
                                "+" => "add",
                                "-" => "sub",
                                "*" => "mul",
                                "/" => "div",
                                "==" => "eq",
                                "!=" => "neq",
                                "<" => "lt",
                                ">" => "gt",
                                "<=" => "le",
                                ">=" => "ge",
                                "+=" => "add_assign",
                                "-=" => "sub_assign",
                                "*=" => "mul_assign",
                                "/=" => "div_assign",
                                _ => "unknown_op",
                            };
                            
                            // Transform: obj + other -> Class_operator_add(obj, other)
                            out_tokens.push(Token::Identifier(format!("{}_operator_{}", class_with_namespace, operator_name)));
                            out_tokens.push(Token::Symbol("(".to_string()));
                            out_tokens.push(Token::Identifier(left_operand.clone()));
                            out_tokens.push(Token::Symbol(",".to_string()));
                            out_tokens.push(tokens[i + 2].clone()); // right operand
                            out_tokens.push(Token::Symbol(")".to_string()));
                            
                            i += 3; // Skip past the binary operation
                            continue;
                        }
                        
                        // Check for unary operators: obj++, ++obj, obj--, --obj
                        if matches!(operator.as_str(), "++" | "--") {
                            if DEBUG {println!("DEBUG: Found postfix unary operator: {}{}", left_operand, operator);}
                            
                            let class_with_namespace = class_names.get(&var.type_).unwrap_or(&var.type_);
                            let operator_name = match operator.as_str() {
                                "++" => "increment",
                                "--" => "decrement",
                                _ => "unknown_op",
                            };
                            
                            // Transform: obj++ -> Class_operator_increment(obj)
                            out_tokens.push(Token::Identifier(format!("{}_operator_{}", class_with_namespace, operator_name)));
                            out_tokens.push(Token::Symbol("(".to_string()));
                            out_tokens.push(Token::Identifier(left_operand.clone()));
                            out_tokens.push(Token::Symbol(")".to_string()));
                            
                            i += 2; // Skip past the unary operation
                            continue;
                        }
                    }
                }
                
                // Handle method calls (existing logic)
                if i + 3 < tokens.len() {
                    if let (Token::Symbol(dot), Token::Identifier(method_name), Token::Symbol(left_paren)) = 
                        (&tokens[i + 1], &tokens[i + 2], &tokens[i + 3]) {
                        
                        if dot == "." && left_paren == "(" {
                            if DEBUG {println!("DEBUG: Found method call: {}.{}(", left_operand, method_name);}
                            
                            // Find closing parenthesis and collect parameters
                            let mut paren_level = 1;
                            let mut p = i + 4;
                            let mut call_params: Vec<Token> = Vec::new();
                            
                            while p < tokens.len() && paren_level > 0 {
                                match &tokens[p] {
                                    Token::Symbol(s) if s == "(" => {
                                        paren_level += 1;
                                        call_params.push(tokens[p].clone());
                                    }
                                    Token::Symbol(s) if s == ")" => {
                                        paren_level -= 1;
                                        if paren_level > 0 {
                                            call_params.push(tokens[p].clone());
                                        }
                                    }
                                    _ => call_params.push(tokens[p].clone()),
                                }
                                p += 1;
                            }
                            
                            let class_with_namespace = class_names.get(&var.type_).unwrap_or(&var.type_);
                            
                            // Transform: obj.method(params) -> Class_method(obj, params)
                            out_tokens.push(Token::Identifier(format!("{}_{}", class_with_namespace, method_name)));
                            out_tokens.push(Token::Symbol("(".to_string()));
                            out_tokens.push(Token::Identifier(left_operand.clone()));
                            
                            if !call_params.is_empty() {
                                out_tokens.push(Token::Symbol(",".to_string()));
                                out_tokens.extend(call_params);
                            }
                            
                            out_tokens.push(Token::Symbol(")".to_string()));
                            
                            i = p;
                            continue;
                        }
                    }
                }
            }
        }
        
        // Handle prefix unary operators: ++obj, --obj
        if let Token::Symbol(operator) = &tokens[i] {
            if matches!(operator.as_str(), "++" | "--") && i + 1 < tokens.len() {
                if let Token::Identifier(operand) = &tokens[i + 1] {
                    if let Some(var) = variables.iter().find(|v| &v.name == operand) {
                        if DEBUG {println!("DEBUG: Found prefix unary operator: {}{}", operator, operand);}
                        
                        let class_with_namespace = class_names.get(&var.type_).unwrap_or(&var.type_);
                        let operator_name = match operator.as_str() {
                            "++" => "increment",
                            "--" => "decrement",
                            _ => "unknown_op",
                        };
                        
                        // Transform: ++obj -> Class_operator_increment(obj)
                        out_tokens.push(Token::Identifier(format!("{}_operator_{}", class_with_namespace, operator_name)));
                        out_tokens.push(Token::Symbol("(".to_string()));
                        out_tokens.push(Token::Identifier(operand.clone()));
                        out_tokens.push(Token::Symbol(")".to_string()));
                        
                        i += 2; // Skip past the prefix operation
                        continue;
                    }
                }
            }
        }
        
        // Handle namespace resolution: namespace::class or namespace::function
        if let Token::Identifier(first_part) = &tokens[i] {
            if i + 2 < tokens.len() {
                if let (Token::Symbol(scope_res), Token::Identifier(second_part)) = (&tokens[i + 1], &tokens[i + 2]) {
                    if scope_res == "::" {
                        if DEBUG {println!("DEBUG: Found namespace resolution: {}::{}", first_part, second_part);}
                        
                        // Replace namespace::identifier with namespace_identifier
                        out_tokens.push(Token::Identifier(format!("{}_{}", first_part, second_part)));
                        i += 3; // Skip past the namespace resolution
                        continue;
                    }
                }
            }
        }
        
        // Copy non-special tokens as is
        out_tokens.push(tokens[i].clone());
        i += 1;
    }

    if DEBUG {println!("DEBUG: parse_function_calls_with_operators completed, {} input tokens -> {} output tokens", 
             tokens.len(), out_tokens.len())};
    out_tokens
}

fn parse_variables(tokens: &[Token]) -> Vec<Variable> {
    if DEBUG {println!("DEBUG: Starting parse_variables with {} tokens", tokens.len());}
    let mut vars = Vec::new();
    let mut i = 0;

    while i + 2 < tokens.len() {
        if let Token::Identifier(type_) = &tokens[i] {
            if let Token::Identifier(name) = &tokens[i + 1] {
                if let Token::Symbol(sym) = &tokens[i + 2] {
                    if sym == ";" {
                        // Vector e;
                        if DEBUG {
                            println!("DEBUG: Found variable: {} {}", type_, name);
                        }
                        vars.push(Variable {
                            name: name.clone(),
                            type_: type_.clone(),
                        });
                        i += 3;
                        continue;
                    } else if sym == "=" {
                        // Vector e = ...;
                        if DEBUG {
                            println!(
                                "DEBUG: Found variable with assignment: {} {}",
                                type_, name
                            );
                        }
                        vars.push(Variable {
                            name: name.clone(),
                            type_: type_.clone(),
                        });

                        // Skip to the semicolon after the assignment expression
                        let mut j = i + 3;
                        while j < tokens.len() {
                            if let Token::Symbol(s) = &tokens[j] {
                                if s == ";" {
                                    break;
                                }
                            }
                            j += 1;
                        }
                        i = j + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }



    if DEBUG {println!("DEBUG: parse_variables completed, found {} variables", vars.len());}
    vars
}

fn replace_class_tokens(tokens: Vec<Token>, classes: &Vec<Class>) -> Vec<Token> {
    let mut out_tokens = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // Handle namespace declarations
        if let Some((namespace_name, content_start)) = parse_namespace_declaration(&tokens, i) {
            if DEBUG {println!("DEBUG: Processing namespace: {}", namespace_name);}
            
            let namespace_end = find_namespace_end(&tokens, content_start);
            
            // Process content inside namespace but dont output namespace wrapper
            let namespace_content = &tokens[content_start..namespace_end-1]; // exclude closing brace
            let processed_content = replace_class_tokens(namespace_content.to_vec(), classes);
            
            out_tokens.extend(processed_content);
            i = namespace_end;
            continue;
        }
        
        if let Token::Identifier(token_name) = &tokens[i] {
            if token_name == "class" {
                // Find class name
                if let Some(Token::Identifier(class_name)) = tokens.get(i + 1) {
                    // Check if this class is in our list
                    if classes.iter().any(|c| &c.name == class_name) {
                        // Skip tokens until closing brace of class
                        i += 2; // Skip "class ClassName"
                        let mut brace_level = 0;

                        // Find {
                        if let Some(Token::Symbol(s)) = tokens.get(i) {
                            if s == "{" {
                                brace_level += 1;
                                i += 1;
                            }
                        }

                        while i < tokens.len() && brace_level > 0 {
                            match &tokens[i] {
                                Token::Symbol(s) if s == "{" => brace_level += 1,
                                Token::Symbol(s) if s == "}" => brace_level -= 1,
                                _ => {}
                            }
                            i += 1;
                        }

                        // Insert generated class code as tokens
                        let generated_code = classes
                            .iter()
                            .find(|c| &c.name == class_name)
                            .unwrap()
                            .to_string();
                        
                        let generated_tokens = tokenize(&generated_code);
                        for token in generated_tokens {
                            if !matches!(token, Token::Eof) {
                                out_tokens.push(token);
                            }
                        }

                        continue;
                    }
                }
            }
        }

        // Copy non-class tokens
        out_tokens.push(tokens[i].clone());
        i += 1;
    }

    out_tokens
}

// Driver
pub fn compile(src: &str) -> String {
    compile_with_context(src, &mut HashMap::new())
}

fn compile_with_context(src: &str, known_classes: &mut HashMap<String, String>) -> String {
    if DEBUG {println!("DEBUG: Starting compilation with {} known classes", known_classes.len());}
    let mut tokens = tokenize(src);

    if DEBUG {println!("DEBUG: Tokenized source into {} tokens", tokens.len());}
    
    if DEBUG {println!("{:?}", &tokens);}

    // First pass: collect class names and namespaces from THIS file before processing imports
    let mut current_namespace: Option<String> = None;
    let mut i = 0;
    
    while i < tokens.len() {
        // Check for namespace declaration
        if let Some((namespace_name, content_start)) = parse_namespace_declaration(&tokens, i) {
            current_namespace = Some(namespace_name.clone());
            if DEBUG {println!("DEBUG: Entering namespace: {}", namespace_name);}
            i = content_start;
            continue;
        }
        
        // Check for end of namespace
        if current_namespace.is_some() {
            if let Token::Symbol(brace) = &tokens[i] {
                if brace == "}" {
                    if DEBUG {println!("DEBUG: Exiting namespace: {:?}", current_namespace);}
                    current_namespace = None;
                    i += 1;
                    continue;
                }
            }
        }
        
        // Check for class declaration
        if let Token::Identifier(keyword) = &tokens[i] {
            if keyword == "class" {
                if let Some(Token::Identifier(class_name)) = tokens.get(i + 1) {
                    let full_class_name = match &current_namespace {
                        Some(ns) => format!("{}_{}", ns, class_name),
                        None => class_name.clone(),
                    };
                    
                    if DEBUG {println!("DEBUG: Found class {} (full name: {})", class_name, full_class_name);}
                    known_classes.insert(class_name.clone(), full_class_name);
                }
            }
        }
        
        i += 1;
    }

    if DEBUG {println!("DEBUG: After local scan, total known classes: {}", known_classes.len());}

    // Process imports
    i = 0;
    while i < tokens.len() {
        if let Token::Symbol(tag) = &tokens[i] {
            if tag == "#" {
                if let Token::Identifier(import) = &tokens[i + 1] {
                    if import == "import" {
                        if let Token::Symbol(left_angle) = &tokens[i + 2] {
                            if left_angle == "<" {
                                i += 3;
                                let mut filename: String = String::new();

                                // Find the end of the filename and remember the index of >
                                let mut end_of_import = i;
                                while let Some(token) = tokens.get(end_of_import) {
                                    match token {
                                        Token::Symbol(right_angle) if right_angle == ">" => {
                                            break;
                                        }
                                        Token::Identifier(inside) | Token::Symbol(inside) => {
                                            filename.push_str(inside);
                                        }
                                        _ => break,
                                    }
                                    end_of_import += 1;
                                }

                                // Actually load the file and tokenize it
                                let file_content = std::fs::read_to_string(&filename)
                                    .unwrap_or_else(|_| panic!("Failed to read import file: {}", filename));

                                // Compile imported file with the current known classes context
                                let imported_tokens = compile_with_context(&file_content, known_classes);

                                // Replace the whole `# import < ... >` span with the compiled code
                                tokens.splice(i - 3..=end_of_import, tokenize(imported_tokens.as_str()));

                                // i now points just after the inserted tokens
                                continue;
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }
    
    if DEBUG {println!("{:?}", tokens);}
    if DEBUG {println!("DEBUG: After import processing, known classes: {:?}", known_classes);}

    // Parse class definitions from current file with namespace support
    let mut classes: Vec<Class> = Vec::new();
    current_namespace = None;
    i = 0;
    
    while i < tokens.len() {
        if i % 100 == 0 {
            if DEBUG {println!("DEBUG: compile - processing token {} of {}", i, tokens.len());}
        }
        
        // Handle namespace declarations
        if let Some((namespace_name, content_start)) = parse_namespace_declaration(&tokens, i) {
            current_namespace = Some(namespace_name);
            i = content_start;
            continue;
        }
        
        // Handle end of namespace
        if current_namespace.is_some() {
            if let Token::Symbol(brace) = &tokens[i] {
                if brace == "}" {
                    current_namespace = None;
                    i += 1;
                    continue;
                }
            }
        }
        
        if let Token::Identifier(token_name) = &tokens[i] {
            if token_name == "class" {
                if DEBUG {println!("DEBUG: Found class keyword at token {}", i);}
                
                if let Some(Token::Identifier(class_name)) = tokens.get(i + 1) {
                    if DEBUG {println!("DEBUG: Class name: {} (namespace: {:?})", class_name, current_namespace);}
                    
                    let mut class = Class {
                        name: class_name.clone(),
                        namespace: current_namespace.clone(),
                        functions: Vec::new(),
                        variables: Vec::new(),
                        operators: Vec::new(),
                    };

                    // look for { to start class body
                    let mut j = i + 2;
                    if let Some(Token::Symbol(s)) = tokens.get(j) {
                        if s == "{" {
                            if DEBUG {println!("DEBUG: Found class opening brace at token {}", j);}
                            j += 1;
                            let mut brace_level = 1;

                            let mut class_body_tokens: Vec<Token> = Vec::new();

                            while j < tokens.len() && brace_level > 0 {
                                match &tokens[j] {
                                    Token::Symbol(s) if s == "{" => {
                                        brace_level += 1;
                                        class_body_tokens.push(tokens[j].clone());
                                    }
                                    Token::Symbol(s) if s == "}" => {
                                        brace_level -= 1;
                                        if brace_level > 0 {
                                            class_body_tokens.push(tokens[j].clone());
                                        }
                                    }
                                    _ => class_body_tokens.push(tokens[j].clone()),
                                }
                                j += 1;
                            }

                            if DEBUG {println!("DEBUG: Class body extracted, {} tokens collected", class_body_tokens.len());}
                            
                            // Parse functions and operators
                            let (functions, operators) = parse_functions_with_operators(&class_body_tokens, class.name.clone(), current_namespace.clone());
                            class.functions = functions;
                            class.operators = operators;
                            class.variables = parse_variables(&class_body_tokens);
                            
                            if DEBUG {println!("DEBUG: Class {} parsed with {} functions, {} operators, and {} variables", 
                                class_name, class.functions.len(), class.operators.len(), class.variables.len())};
                        }
                    }

                    classes.push(class);
                    i = j;
                    continue;
                }
            }
        }

        i += 1;
    }

    if DEBUG {println!("DEBUG: Class parsing completed, found {} classes in current file", classes.len());}

    // Transform function calls and operators using all known class names
    tokens = parse_function_calls_with_operators(tokens, known_classes.clone());

    // Replace class definitions with generated C code
    tokens = replace_class_tokens(tokens, &classes);

    let final_code = detokenize(&tokens);
    final_code
}