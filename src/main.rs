use z_lang::{compile, DEBUG};
use std::fs;
use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut gcc_args: Vec<String> = Vec::new();

    let source = fs::read_to_string("main.z");
    let c_code = compile(source.unwrap().as_str());
    if DEBUG {println!("{}", c_code)};

    let mut path: String = "out".to_string();
    let mut main: String = "out".to_string();
    for (i, arg) in args.iter().enumerate() {
        if i == 0 {
            continue;
        }

        if arg.ends_with(".z") {
            if arg == "main.z" {
                main = arg.clone();
                continue;
            }

            gcc_args.push(arg.replace(".z", ".c"));
            continue;
        }

        if arg == "-o" {
            path = args[i + 1].clone();
        }

        gcc_args.push(arg.to_string());
    }

    gcc_args.push(main.clone() + ".c");

    println!("{:?}", gcc_args);

    let _ = fs::write(main + ".c", c_code);
    let gcc_output = Command::new("gcc").args(gcc_args).output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&gcc_output.stdout);
    if stdout == "".to_string() {
        return;
    }
    println!("GCC:\n{}", stdout);    
}
