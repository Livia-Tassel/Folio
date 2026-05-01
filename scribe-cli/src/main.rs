//! scribe-cli: command-line harness for fixture testing.
//!
//! Usage:
//!   scribe-cli <input.md> -o <output.docx> [--reference-doc <ref.docx>]
//!   scribe-cli <input.md> -o <output.docx> [--theme <name>]

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut reference_doc: Option<PathBuf> = None;
    let mut theme: Option<String> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output = iter.next().map(PathBuf::from);
            }
            "--reference-doc" => {
                reference_doc = iter.next().map(PathBuf::from);
            }
            "--theme" => {
                theme = iter.next().cloned();
            }
            "--list-themes" => {
                for name in scribe_core::list_builtin_themes() {
                    println!("{name}");
                }
                return ExitCode::SUCCESS;
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

    if reference_doc.is_some() && theme.is_some() {
        eprintln!("error: --reference-doc and --theme are mutually exclusive");
        return ExitCode::FAILURE;
    }

    let Some(input) = input else {
        eprintln!("error: no input file provided");
        print_help();
        return ExitCode::FAILURE;
    };

    let output = output.unwrap_or_else(|| input.with_extension("docx"));

    let template = match (reference_doc, theme) {
        (None, None) => None,
        (Some(path), None) => match scribe_core::Template::from_reference_doc(&path) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!(
                    "error: failed to load reference doc {}: {e}",
                    path.display()
                );
                return ExitCode::FAILURE;
            }
        },
        (None, Some(name)) => match scribe_core::Template::builtin(&name) {
            Ok(t) => Some(t),
            Err(e) => {
                let known = scribe_core::list_builtin_themes().join(", ");
                eprintln!("error: {e} (known themes: {known})");
                return ExitCode::FAILURE;
            }
        },
        (Some(_), Some(_)) => unreachable!(),
    };

    match scribe_core::convert_file_with_template(&input, &output, template.as_ref()) {
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
    println!("    scribe-cli <INPUT.md> [-o OUTPUT.docx] [--reference-doc REF.docx]");
    println!("    scribe-cli <INPUT.md> [-o OUTPUT.docx] [--theme NAME]\n");
    println!("OPTIONS:");
    println!("    -o, --output <path>            Output path (default: <input>.docx)");
    println!("    --reference-doc <path>         Use a reference .docx for styles");
    println!("    --theme <name>                 Use a built-in theme (mutually exclusive with --reference-doc)");
    println!("    --list-themes                  List available built-in themes");
    println!("    -h, --help                     Show this help");
    println!("    -V, --version                  Show version");
}
