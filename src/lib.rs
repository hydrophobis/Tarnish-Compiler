mod tokenizer;
use std::{fmt::format, vec};

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
    variables: Vec<Variable>,
    functions: Vec<Function>
}

impl ToString for Class {
    fn to_string(&self) -> String {
        let mut s = format!("typedef struct {{ ");

        for var in &self.variables {
            s.push_str(var.to_string().as_str());
        }

        s.push_str(format!(" }} {};\n", &self.name).as_str());

        for func in &self.functions {
            s.push_str(func.to_string().as_str());
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
        format!("{} {}_{}({} self, {}){{{}}}", self.return_type, self.class_name, self.name, self.class_name, self.params.join(", "), joined)
    }
}

fn parse_functions(tokens: &[Token], class: String) -> Vec<Function> {
    if DEBUG {println!("DEBUG: Starting parse_functions with {} tokens", tokens.len());}
    let mut functions = Vec::new();
    let mut i = 0;

    while i + 2 < tokens.len() {
        if DEBUG {println!("DEBUG: parse_functions - checking token {} of {}", i, tokens.len());}
        
        // look for return_type identifier '('
        if let Token::Identifier(ret_type) = &tokens[i] {
            if let Token::Identifier(name) = &tokens[i + 1] {
                if let Token::Symbol(sym) = &tokens[i + 2] {
                    if sym == "(" {
                        if DEBUG {println!("DEBUG: Found function: {} {}", ret_type, name);}
                        
                        // parse params until ')'
                        let mut params = Vec::new();
                        let mut p = i + 3;
                        
                        if DEBUG {println!("DEBUG: Parsing parameters starting at token {}", p);}
                        
                        // Handle empty parameter list
                        if p < tokens.len() {
                            if let Token::Symbol(sym) = &tokens[p] {
                                if sym == ")" {
                                    if DEBUG {println!("DEBUG: Empty parameter list");}
                                    p += 1; // move past )
                                }
                            }
                        }
                        
                        // Parse parameters if not empty
                        while p + 1 < tokens.len() {
                            // Check if we hit closing paren first
                            if let Token::Symbol(sym) = &tokens[p] {
                                if sym == ")" {
                                    if DEBUG {println!("DEBUG: Found closing paren at token {}", p);}
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
                                        if DEBUG {println!("DEBUG: Found parameter: {}", param);}
                                        params.push(param);
                                        p += 2; // move past type and name
                                        continue;
                                    }
                                }
                            }
                            
                            // If we can't parse a parameter, advance by 1 and try again
                            p += 1;
                            
                            // Safety check to prevent infinite loop
                            if p >= tokens.len() {
                                if DEBUG {println!("DEBUG: Reached end of tokens while parsing parameters");}
                                break;
                            }
                        }

                        if DEBUG {println!("DEBUG: Finished parsing parameters, looking for function body at token {}", p);}

                        // Find the opening brace for function body
                        while p < tokens.len() {
                            if let Token::Symbol(s) = &tokens[p] {
                                if s == "{" {
                                    if DEBUG {println!("DEBUG: Found opening brace at token {}", p);}
                                    break;
                                }
                            }
                            p += 1;
                        }

                        // parse body tokens after '{'
                        let mut body_tokens = Vec::new();
                        if p < tokens.len() {
                            if let Token::Symbol(s) = &tokens[p] {
                                if s == "{" {
                                    if DEBUG {println!("DEBUG: Parsing function body");}
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
                                    if DEBUG {println!("DEBUG: Function body parsed, {} tokens collected", body_tokens.len());}
                                    i = b; // advance main loop past the function
                                } else {
                                    if DEBUG {println!("DEBUG: Expected opening brace but found something else");}
                                    i += 1;
                                    continue;
                                }
                            }
                        } else {
                            if DEBUG {println!("DEBUG: Reached end of tokens looking for function body");}
                            break;
                        }

                        functions.push(Function {
                            class_name: class.clone(),
                            name: name.clone(),
                            return_type: ret_type.clone(),
                            params,
                            body_tokens,
                        });
                        
                        if DEBUG {println!("DEBUG: Added function {} to list", name);}
                        continue;
                    }
                }
            }
        }
        i += 1;
    }

    if DEBUG {println!("DEBUG: parse_functions completed, found {} functions", functions.len());}
    functions
}

// Separate function to collect all variables from the entire token stream
fn collect_all_variables(tokens: &[Token], class_names: &[String]) -> Vec<Variable> {
    if DEBUG {println!("DEBUG: Collecting all variables from {} tokens", tokens.len());}
    let mut variables = Vec::new();
    let mut i = 0;

    while i + 2 < tokens.len() {
        // Look for: TypeName identifier ;
        if let Token::Identifier(type_name) = &tokens[i] {
            if let Token::Identifier(var_name) = &tokens[i + 1] {
                if let Token::Symbol(semicolon) = &tokens[i + 2] {
                    if semicolon == ";" {
                        // Check if this type is one of our known classes
                        if class_names.contains(type_name) {
                            if DEBUG {println!("DEBUG: Found class variable: {} {}", type_name, var_name);}
                            variables.push(Variable {
                                name: var_name.clone(),
                                type_: type_name.clone(),
                            });
                        }
                        i += 3;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }

    if DEBUG {println!("DEBUG: Found {} variables total", variables.len());}
    for var in &variables {
        if DEBUG {println!("DEBUG: Variable: {} of type {}", var.name, var.type_);}
    }
    
    variables
}

fn parse_function_calls(tokens: Vec<Token>, class_names: Vec<String>) -> Vec<Token> {
    if DEBUG {println!("DEBUG: Starting parse_function_calls with {} tokens and {} classes", tokens.len(), class_names.len());}
    
    // First, collect ALL variables in the program
    let variables = collect_all_variables(&tokens, &class_names);
    
    let mut out_tokens: Vec<Token> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if i % 200 == 0 { // Progress indicator
            if DEBUG {println!("DEBUG: parse_function_calls - processing token {} of {}", i, tokens.len());}
        }

        // Check if current token is a variable name we know about
        if let Token::Identifier(token) = &tokens[i] {
            if let Some(var) = variables.iter().find(|v| &v.name == token) {
                if DEBUG {println!("DEBUG: Found variable usage: {} (type: {})", token, var.type_);}
                
                // Look ahead for method call pattern: varName . methodName (
                if i + 3 < tokens.len() {
                    if let (Token::Symbol(dot), Token::Identifier(method_name), Token::Symbol(left_paren)) = 
                        (&tokens[i + 1], &tokens[i + 2], &tokens[i + 3]) {
                        
                        if dot == "." && left_paren == "(" {
                            if DEBUG {println!("DEBUG: Found method call: {}.{}(", token, method_name);}
                            
                            // Find the closing parenthesis and collect parameters
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
                            
                            if DEBUG {println!("DEBUG: Method call parameters parsed, {} tokens", call_params.len());}
                            
                            // Transform: obj.method(params) -> Class_method(obj, params)
                            out_tokens.push(Token::Identifier(format!("{}_{}", var.type_, method_name)));
                            out_tokens.push(Token::Symbol("(".to_string()));
                            out_tokens.push(Token::Identifier(token.clone())); // the object instance
                            
                            // Add comma and parameters if there are any
                            if !call_params.is_empty() {
                                out_tokens.push(Token::Symbol(",".to_string()));
                                out_tokens.extend(call_params);
                            }
                            
                            out_tokens.push(Token::Symbol(")".to_string()));
                            
                            i = p; // Skip past the entire method call
                            continue;
                        }
                    }
                }
            }
        }
        
        // If we didn't handle this token specially, copy it as-is
        out_tokens.push(tokens[i].clone());
        i += 1;
    }

    if DEBUG {println!("DEBUG: parse_function_calls completed, {} input tokens -> {} output tokens", 
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
                        if DEBUG {println!("DEBUG: Found variable: {} {}", type_, name);}
                        vars.push(Variable {
                            name: name.clone(),
                            type_: type_.clone(),
                        });
                        i += 3;
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
        if let Token::Identifier(token_name) = &tokens[i] {
            if token_name == "class" {
                // Find class name
                if let Some(Token::Identifier(class_name)) = tokens.get(i + 1) {
                    // Check if this class is in our list
                    if classes.iter().any(|c| &c.name == class_name) {
                        // Skip tokens until closing brace of class
                        i += 2; // Skip "class ClassName"
                        let mut brace_level = 0;

                        // Find '{'
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

                        // Insert generated class code as tokens instead of a single string literal
                        let generated_code = classes
                            .iter()
                            .find(|c| &c.name == class_name)
                            .unwrap()
                            .to_string();
                        
                        // Tokenize the generated code and add those tokens
                        let generated_tokens = tokenize(&generated_code);
                        for token in generated_tokens {
                            if !matches!(token, Token::Eof) {
                                out_tokens.push(token);
                            }
                        }

                        continue; // skip the rest of this iteration
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
    compile_with_context(src, &mut Vec::new())
}

fn compile_with_context(src: &str, known_classes: &mut Vec<String>) -> String {
    if DEBUG {println!("DEBUG: Starting compilation with {} known classes", known_classes.len());}
    let mut tokens = tokenize(src);

    if DEBUG {println!("DEBUG: Tokenized source into {} tokens", tokens.len());}
    
    if DEBUG {println!("{:?}", &tokens);}

    // First pass: collect class names from THIS file before processing imports
    let mut local_class_names = Vec::new();
    let mut i = 0;
    while i + 1 < tokens.len() {
        if let Token::Identifier(keyword) = &tokens[i] {
            if keyword == "class" {
                if let Token::Identifier(class_name) = &tokens[i + 1] {
                    if DEBUG {println!("DEBUG: Found class name in current file: {}", class_name);}
                    local_class_names.push(class_name.clone());
                }
            }
        }
        i += 1;
    }

    // Add local classes to known classes
    for class_name in &local_class_names {
        if !known_classes.contains(class_name) {
            known_classes.push(class_name.clone());
        }
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

                                // Find the end of the filename and remember the index of '>'
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

        // Parse class definitions from current file
    let mut classes: Vec<Class> = Vec::new();
    i = 0;
    
    while i < tokens.len() {
        if i % 100 == 0 { // Print progress every 100 tokens
            if DEBUG {println!("DEBUG: compile - processing token {} of {}", i, tokens.len());}
        }
        
        if let Token::Identifier(token_name) = &tokens[i] {
            if token_name == "class" {
                if DEBUG {println!("DEBUG: Found class keyword at token {}", i);}
                
                if let Some(Token::Identifier(class_name)) = tokens.get(i + 1) {
                    if DEBUG {println!("DEBUG: Class name: {}", class_name);}
                    
                    let mut class = Class {
                        name: class_name.clone(),
                        functions: Vec::new(),
                        variables: Vec::new()
                    };

                    // look for '{' to start class body
                    let mut j = i + 2;
                    if let Some(Token::Symbol(s)) = tokens.get(j) {
                        if s == "{" {
                            if DEBUG {println!("DEBUG: Found class opening brace at token {}", j);}
                            j += 1;
                            let mut brace_level = 1;

                            let mut func_tokens: Vec<Token> = Vec::new();

                            while j < tokens.len() && brace_level > 0 {
                                match &tokens[j] {
                                    Token::Symbol(s) if s == "{" => {
                                        brace_level += 1;
                                        func_tokens.push(tokens[j].clone());
                                    }
                                    Token::Symbol(s) if s == "}" => {
                                        brace_level -= 1;
                                        if brace_level > 0 {
                                            func_tokens.push(tokens[j].clone());
                                        }
                                    }
                                    _ => func_tokens.push(tokens[j].clone()),
                                }
                                j += 1;
                            }

                            if DEBUG {println!("DEBUG: Class body extracted, {} tokens collected", func_tokens.len());}
                            
                            let class_body_tokens = &func_tokens;
                            class.functions = parse_functions(class_body_tokens, class.name.clone());
                            class.variables = parse_variables(class_body_tokens);
                            
                            if DEBUG {println!("DEBUG: Class {} parsed with {} functions and {} variables", 
                                class_name, class.functions.len(), class.variables.len())};
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

    // Use all known classes (including imported ones) for function call parsing
    let all_class_names = known_classes.clone();

    // Transform function calls using all known class names
    tokens = parse_function_calls(tokens, all_class_names);

    // Replace class definitions with generated C code
    tokens = replace_class_tokens(tokens, &classes);

    let final_code2 = detokenize(&tokens);

    final_code2
}