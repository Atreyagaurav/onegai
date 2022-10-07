use clap::{Parser, Subcommand};
use colored::Colorize;
use std::time::Instant;

mod ncode;
mod renderer;
mod replacements;
mod speak;
mod translator;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Command to execute
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Build a html page for viewing the file contents
    Combine(renderer::CliArgs),
    /// Replace the terms according to rules on json file
    Replace(replacements::CliArgs),
    /// Translate from Japanese to English
    Translate(translator::CliArgs),
    /// Download a web novel chapter from syosetu into a text file
    Download(ncode::CliArgs),
    /// Use google speak service to speak out the file [experimental]
    Speak(speak::CliArgs),
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    let tool_result = match args.action {
        Action::Combine(args) => renderer::make_html(args),
        Action::Replace(args) => replacements::replace_from_json(args),
        Action::Translate(args) => translator::translate(args),
        Action::Download(args) => ncode::download_ncode(args),
        Action::Speak(args) => speak::gspeak(args),
    };
    let duration = start.elapsed();
    match tool_result {
        Ok(_) => (),
        Err(e) => eprintln!("{}: {}", "Dame".bright_red().bold(), e),
    }
    eprintln!("{}: {:?}", "Time Elapsed".blue().bold(), duration);
}
