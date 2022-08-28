use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    /// Command to execute
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// TODO Run a server that provides a html page for viewing the file contents
    Server {
        /// Ignore Lines match error
        #[clap(short, long, action)]
        ignore_match: bool,
        /// Input files in different languages
        input_files: Vec<PathBuf>,
    },
    /// TODO Replace the terms according to rules on json file
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
        /// Input file in Japanese
        input_file: PathBuf,
        /// Output file in English
        output_file: PathBuf,
    },
}

fn main() {
    let args = Cli::parse();
    match args.action {
        Action::Server {
            ignore_match,
            input_files,
        } => start_server(ignore_match, input_files),
        Action::Replace {
            replacement_json,
            input_file,
            output_file,
        } => replace_from_json(replacement_json, input_file, output_file),
        Action::Translate {
            input_file,
            output_file,
        } => translate(input_file, output_file),
    }
}

fn start_server(ignore_match_err: bool, input_files: Vec<PathBuf>) {
    println!("{} {:?}", ignore_match_err, input_files);
}

fn replace_from_json(replacement_json: PathBuf, input_file: PathBuf, output_file: PathBuf) {
    println!("{:?} {:?} {:?}", replacement_json, input_file, output_file,)
}

fn translate(input_file: PathBuf, output_file: PathBuf) {
    println!("{:?} {:?}", input_file, output_file)
}
