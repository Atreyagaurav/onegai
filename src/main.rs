use clap::{Parser, Subcommand};
use maud::{html, Markup, DOCTYPE};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// Command to execute
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Build a html page for viewing the file contents
    Html {
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
        Action::Html {
            ignore_match,
            input_files,
        } => make_html(ignore_match, input_files),
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

fn make_html(ignore_match_err: bool, input_files: Vec<PathBuf>) {
    let lines: Vec<usize> = input_files
        .iter()
        .map(|f| {
            let file = File::open(f.clone()).expect(&format!(
                "Couldn't open input file {}.",
                f.as_os_str().to_str().unwrap()
            ));
            BufReader::new(file).lines().count()
        })
        .collect();
    if !ignore_match_err {
        if !lines.iter().all(|v| *v == lines[0]) {
            panic!("File lengths doesn't match. {:?}", lines);
        }
    }
    let num_lines: usize = *lines.iter().min().unwrap();

    let readers: Vec<Lines<BufReader<File>>> = input_files
        .iter()
        .map(|fp| {
            let file = File::open(fp.clone()).unwrap();
            let lines = BufReader::new(file).lines();
            return lines;
        })
        .collect();
    println!("{}", get_html(num_lines, readers).into_string());
}

fn get_html(num_lines: usize, mut readers: Vec<Lines<BufReader<File>>>) -> Markup {
    html! {
    (DOCTYPE)
        h3 {"Title"}
        @for i in 1..=num_lines {
            @let linetag=format!("line-{}",i);
            div id=(linetag) {
        p {
            @for reader in readers.iter_mut() {
            @match reader.next() {
                Some(line) => {(line.unwrap())},
                None => {""},
            }
        br;
            }
        }}
        }}
}

fn replace_from_json(replacement_json: PathBuf, input_file: PathBuf, output_file: PathBuf) {
    println!("{:?} {:?} {:?}", replacement_json, input_file, output_file,)
}

fn translate(input_file: PathBuf, _output_file: PathBuf) {
    let file = File::open(input_file).expect("Couldn't open input file.");
    let reader = BufReader::new(file);

    let model = TranslationModelBuilder::new()
        .with_model_type(ModelType::MBart)
        .with_source_languages(vec![Language::Japanese])
        .with_target_languages(vec![Language::English])
        .create_model()
        .unwrap();

    let mut line: String;
    for input_line in reader.lines() {
        line = input_line.unwrap();
        if line.trim() != "" {
            let output2 = model.translate(&[line.clone()], Language::Japanese, Language::English);
            for sentence in output2 {
                println!("{}", sentence.join(" "));
            }
        } else {
            println!("");
        }
    }
}
