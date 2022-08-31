use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Instant;

mod renderer;
mod replacements;
mod translator;

#[derive(Parser)]
struct Cli {
    /// Command to execute
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Build a html page for viewing the file contents
    Combine {
        /// Ignore Lines match error
        #[clap(short, long, action)]
        ignore_match: bool,
        /// Input files in different languages
        input_files: Vec<PathBuf>,
    },
    /// Replace the terms according to rules on json file
    Replace {
        /// Replacement Json
        #[clap(short, long)]
        replacement_json: PathBuf,
        /// Input file
        input_file: PathBuf,
        /// Output file
        output_file: PathBuf,
    },
    /// Translate from Japanese to English
    Translate {
        /// Number of starting lines to skip translation
        #[clap(short, long, default_value = "0")]
        skip_lines: usize,
        /// Append to the output file instead of overwriting
        #[clap(short, long, action)]
        append: bool,
        /// Input file in Japanese
        input_file: PathBuf,
        /// Output file in English
        output_file: PathBuf,
    },
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    match args.action {
        Action::Combine {
            ignore_match,
            input_files,
        } => renderer::make_html(ignore_match, input_files),
        Action::Replace {
            replacement_json,
            input_file,
            output_file,
        } => replacements::replace_from_json(replacement_json, input_file, output_file),
        Action::Translate {
            skip_lines,
            append,
            input_file,
            output_file,
        } => translator::translate(skip_lines, append, input_file, output_file),
    }
    let duration = start.elapsed();
    eprintln!("Time Elapsed: {:?}", duration);
}
