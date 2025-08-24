// src/tokenizer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(String),
    StringLit(String),
    CharLit(String),
    Symbol(String),   // operators and punctuators, multi-char if needed
    Comment(String),  // keeps //... or /* ... */
    Newline,
    Eof,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let len = input.len();
    let s = input;

    // Operators / punctuators (put longest first)
    let mut ops = vec![
        ">>=", "<<=", "==", "!=", "<=", ">=", "->", "++", "--", "&&", "||", "+=", "-=", "*=",
        "/=", "%=", "&=", "|=", "^=", "<<", ">>", "::", "=>"
    ];
    // single-char will be matched by fallback
    ops.sort_by(|a, b| b.len().cmp(&a.len()));

    while i < len {
        let ch = s.as_bytes()[i] as char;

        // Newline handling (preserve)
        if ch == '\n' {
            tokens.push(Token::Newline);
            i += 1;
            continue;
        }

        // Skip other whitespace
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        // Comments: //... or /* ... */
        if ch == '/' && i + 1 < len {
            let next = s.as_bytes()[i + 1] as char;
            if next == '/' {
                // line comment
                let start = i;
                i += 2;
                while i < len && (s.as_bytes()[i] as char) != '\n' {
                    i += 1;
                }
                let comment = &s[start..i];
                tokens.push(Token::Comment(comment.to_string()));
                continue;
            } else if next == '*' {
                // block comment
                let start = i;
                i += 2;
                while i + 1 < len && !(s.as_bytes()[i] as char == '*' && s.as_bytes()[i + 1] as char == '/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2; // consume */
                }
                let comment = &s[start..i.min(len)];
                tokens.push(Token::Comment(comment.to_string()));
                continue;
            }
        }

        // Strings and char literals
        if ch == '"' || ch == '\'' {
            let quote = ch;
            let start = i;
            i += 1;
            while i < len {
                let c = s.as_bytes()[i] as char;
                if c == '\\' {
                    // escape: include next char too
                    i += 2;
                    continue;
                }
                if c == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let slice = &s[start..i.min(len)];
            if quote == '"' {
                tokens.push(Token::StringLit(slice.to_string()));
            } else {
                tokens.push(Token::CharLit(slice.to_string()));
            }
            continue;
        }

        // Numbers: hex (0x), floats, decimals
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && (s.as_bytes()[i+1] as char).is_ascii_digit()) {
            let start = i;
            // hex
            if ch == '0' && i + 1 < len && ((s.as_bytes()[i + 1] as char) == 'x' || (s.as_bytes()[i + 1] as char) == 'X') {
                i += 2;
                while i < len && (s.as_bytes()[i] as char).is_ascii_hexdigit() {
                    i += 1;
                }
            } else {
                // decimal / float / exponent
                while i < len && ((s.as_bytes()[i] as char).is_ascii_digit()) {
                    i += 1;
                }
                // fraction
                if i < len && (s.as_bytes()[i] as char) == '.' {
                    i += 1;
                    while i < len && ((s.as_bytes()[i] as char).is_ascii_digit()) {
                        i += 1;
                    }
                }
                // exponent
                if i < len {
                    let c = s.as_bytes()[i] as char;
                    if c == 'e' || c == 'E' {
                        i += 1;
                        if i < len {
                            let sign = s.as_bytes()[i] as char;
                            if sign == '+' || sign == '-' {
                                i += 1;
                            }
                        }
                        while i < len && ((s.as_bytes()[i] as char).is_ascii_digit()) {
                            i += 1;
                        }
                    }
                }
            }
            tokens.push(Token::Number(s[start..i.min(len)].to_string()));
            continue;
        }

        // Identifier or keyword-like token
        if ch == '_' || ch.is_alphabetic() {
            let start = i;
            i += 1;
            while i < len {
                let c = s.as_bytes()[i] as char;
                if c == '_' || c.is_alphanumeric() {
                    i += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token::Identifier(s[start..i].to_string()));
            continue;
        }

        // Operators / multi-char symbols (longest-first)
        let mut matched_op = None;
        for &op in &ops {
            if i + op.len() <= len && &s[i..i + op.len()] == op {
                matched_op = Some(op);
                break;
            }
        }
        if let Some(op) = matched_op {
            tokens.push(Token::Symbol(op.to_string()));
            i += op.len();
            continue;
        }

        // Single-char symbol/punctuator fallback
        tokens.push(Token::Symbol(ch.to_string()));
        i += 1;
    }

    tokens.push(Token::Eof);
    tokens
}

pub fn detokenize(tokens: &[Token]) -> String {
    let mut output = String::new();
    let mut prev_token: Option<&Token> = None;

    for token in tokens {
        if matches!(token, Token::Eof) {
            continue; // skip EOF
        }

        // Handle spacing
        if let Some(prev) = prev_token {
            if needs_space(prev, token) {
                output.push(' ');
            }
        }

        match token {
            Token::Identifier(s)
            | Token::Number(s)
            | Token::StringLit(s)
            | Token::CharLit(s)
            | Token::Comment(s)
            | Token::Symbol(s) => {
                output.push_str(s);
            }
            Token::Newline => {
                output.push('\n');
            }
            Token::Eof => {} // already skipped
        }

        prev_token = Some(token);
    }

    output
}

fn needs_space(prev: &Token, current: &Token) -> bool {
    use Token::*;
    match (prev, current) {
        // Never space around newlines or comments
        (Newline, _) | (_, Newline) | (Comment(_), _) => false,

        // Symbols that should never have spaces around them
        (Symbol(a), Symbol(b)) => {
            match (a.as_str(), b.as_str()) {
                // No space around parentheses, brackets, member access
                ("(", _) | (_, ")") | ("[", _) | (_, "]") => false,
                (".", _) | (_, ".") => false,
                ("->", _) | (_, "->") => false,
                // No space after opening angle bracket or before closing
                ("<", _) | (_, ">") => false,
                // No space around semicolons and commas
                (";", _) | (_, ";") | (",", _) | (_, ",") => false,
                // Space around most other operators
                _ => true,
            }
        }

        // Identifier followed by symbol
        (Identifier(_), Symbol(s)) => {
            match s.as_str() {
                // No space before these symbols
                "(" | "[" | "." | "->" | ";" | "," | ">" => false,
                _ => true,
            }
        }

        // Symbol followed by identifier
        (Symbol(s), Identifier(_)) => {
            match s.as_str() {
                // No space after these symbols
                "(" | "[" | "." | "->" | "!" | "~" | "*" | "&" | "+" | "-" | "<" => false,
                _ => true,
            }
        }

        // Symbol followed by number
        (Symbol(s), Number(_)) => {
            match s.as_str() {
                // No space after these symbols when followed by numbers
                "(" | "[" | "." | "->" | "!" | "~" | "*" | "&" | "+" | "-" | "<" => false,
                _ => true,
            }
        }

        // Number followed by symbol
        (Number(_), Symbol(s)) => {
            match s.as_str() {
                // No space before these symbols
                "(" | "[" | "." | "->" | ";" | "," | ">" | ")" | "]" => false,
                _ => true,
            }
        }

        // Always space between identifiers/numbers
        (Identifier(_), Identifier(_)) => true,
        (Identifier(_), Number(_)) => true,
        (Number(_), Identifier(_)) => true,
        (Number(_), Number(_)) => true,

        // String/char literals always spaced
        (StringLit(_), _) | (_, StringLit(_)) | (CharLit(_), _) | (_, CharLit(_)) => true,

        // Default
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{tokenize, Token};
    
    #[test]
    fn test_basic_detokenization() {
        let input = "int main() { return 0; }";
        let tokens = tokenize(input);
        let output = detokenize(&tokens);
        assert_eq!(output, "int main() { return 0; }");
    }
    
    #[test]
    fn test_member_access() {
        let tokens = vec![
            Token::Identifier("obj".to_string()),
            Token::Symbol(".".to_string()),
            Token::Identifier("member".to_string()),
        ];
        let output = detokenize(&tokens);
        assert_eq!(output, "obj.member");
    }
    
    #[test]
    fn test_function_call() {
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::Symbol("(".to_string()),
            Token::Identifier("arg".to_string()),
            Token::Symbol(",".to_string()),
            Token::Number("42".to_string()),
            Token::Symbol(")".to_string()),
        ];
        let output = detokenize(&tokens);
        assert_eq!(output, "func(arg, 42)");
    }
    
    #[test]
    fn test_arithmetic() {
        let tokens = vec![
            Token::Identifier("a".to_string()),
            Token::Symbol("=".to_string()),
            Token::Identifier("b".to_string()),
            Token::Symbol("+".to_string()),
            Token::Number("1".to_string()),
        ];
        let output = detokenize(&tokens);
        assert_eq!(output, "a = b + 1");
    }

    #[test]
    fn test_include_directive() {
        let tokens = vec![
            Token::Symbol("#".to_string()),
            Token::Identifier("include".to_string()),
            Token::Symbol("<".to_string()),
            Token::Identifier("stdio".to_string()),
            Token::Symbol(".".to_string()),
            Token::Identifier("h".to_string()),
            Token::Symbol(">".to_string()),
        ];
        let output = detokenize(&tokens);
        assert_eq!(output, "#include <stdio.h>");
    }

    #[test] 
    fn test_struct_member_access() {
        let tokens = vec![
            Token::Identifier("self".to_string()),
            Token::Symbol(".".to_string()),
            Token::Identifier("f".to_string()),
            Token::Symbol("=".to_string()),
            Token::Number("1".to_string()),
        ];
        let output = detokenize(&tokens);
        assert_eq!(output, "self.f = 1");
    }
}