//! scribe-cli: command-line harness for fixture testing.
//!
//! Usage:
//!   scribe-cli <input.md> -o <output.docx>

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = iter.next().map(PathBuf::from);
            }
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("scribe-cli {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with('-') && input.is_none() => {
                input = Some(PathBuf::from(other));
            }
            other => {
                eprintln!("error: unexpected argument: {other}");
                print_help();
                return ExitCode::FAILURE;
            }
        }
    }

    let Some(input) = input else {
        eprintln!("error: no input file provided");
        print_help();
        return ExitCode::FAILURE;
    };

    let output = output.unwrap_or_else(|| input.with_extension("docx"));

    match scribe_core::convert_file(&input, &output) {
        Ok(()) => {
            println!("wrote {}", output.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!("scribe-cli — Markdown → .docx converter\n");
    println!("USAGE:");
    println!("    scribe-cli <INPUT.md> [-o OUTPUT.docx]\n");
    println!("OPTIONS:");
    println!("    -o, --output <path>   Output path (default: <input>.docx)");
    println!("    -h, --help            Show this help");
    println!("    -V, --version         Show version");
}
