use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::time::Instant;

mod ncode;
mod renderer;
mod replacements;
mod speak;
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
        /// Title of the Generated HTML
        #[clap(short, long, default_value = "Onegai | Combined Content")]
        title: String,
        /// Make a simple html without any contents check
        ///
        /// When your translations are not made with onegai tools, the
        /// lines might not match, this will help you generate simple
        /// html to see where the descripancy is at.
        #[clap(short, long, action)]
        simple_html: bool,
        /// Output file (html) to save combined page.
        #[clap(short, long, default_value = "onegai-combined.html")]
        output_file: PathBuf,
        /// Input files in different languages
        #[clap(min_values(1))] // this didn't work.
        input_files: Vec<PathBuf>,
    },
    /// Replace the terms according to rules on json file
    Replace {
        /// Replacement Threshold for names without honorifics
        ///
        /// For example, threshold of 3 means names with single kanji
        /// won't be replaced if it comes without honorifics. To make
        /// it easier to decide on this the length of strings are
        /// shown in square brackets after them.
        #[clap(short, long, default_value = "3")]
        threshold: usize,
        /// Replacement Json
        ///
        /// Replacement Json must have 3 fields, rules honorifics and
        /// contents.  `rules' contains the order of replacement and
        /// extra informations, honorifics are list of honorifics to
        /// cycle through for each name, and contents are the
        /// replacement contents.
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
        /// Line content pattern to skip from translation
        ///
        /// Use this for any tags, comments, images etc that you want
        /// to keep as it is.
        #[clap(short, long, default_value = "^<.*>$")]
        pattern_skip: String,
        /// Append to the output file
        #[clap(short, long, action)]
        append: bool,
        /// Overwrite the output file
        #[clap(short, long, action)]
        overwrite: bool,
        /// Resume the previous Translation
        #[clap(short, long, action)]
        resume: bool,
        /// Input file in Japanese
        input_file: PathBuf,
        /// Output file in English
        output_file: PathBuf,
    },
    /// Download a web novel chapter from syosetu into a text file
    Download {
        /// Chapter url
        ///
        /// Chapter url should be from syosetu.com. Examples of
        /// supported urls are:
        /// https://ncode.syosetu.com/n2267be/561/,
        /// ncode.syosetu.com/n2267be/561/, n2267be/561/, etc
        ncode_url: String,
        /// Output file to save the chapter
        output_file: PathBuf,
    },
    /// Use google speak service to speak out the file [experimental]
    Speak {
        /// the input file is in Japanese not English
        #[clap(short, long, action)]
        is_jp: bool,
        /// Input file path in English or Japanese
        input_file: PathBuf,
    },
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    let tool_result = match args.action {
        Action::Combine {
            title,
            simple_html,
            output_file,
            input_files,
        } => renderer::make_html(title, simple_html, output_file, input_files),
        Action::Replace {
            threshold,
            replacement_json,
            input_file,
            output_file,
        } => replacements::replace_from_json(threshold, replacement_json, input_file, output_file),
        Action::Translate {
            skip_lines,
            pattern_skip,
            append,
            overwrite,
            resume,
            input_file,
            output_file,
        } => translator::translate(
            skip_lines,
            pattern_skip,
            append,
            overwrite,
            resume,
            input_file,
            output_file,
        ),
        Action::Download {
            ncode_url,
            output_file,
        } => ncode::download_ncode(ncode_url, output_file),
        Action::Speak { is_jp, input_file } => speak::gspeak(input_file, is_jp),
    };
    let duration = start.elapsed();
    match tool_result {
        Ok(_) => (),
        Err(e) => eprintln!("{}: {}", "Dame".bright_red().bold(), e),
    }
    eprintln!("{}: {:?}", "Time Elapsed".blue().bold(), duration);
}
