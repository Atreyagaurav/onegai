use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, LineWriter, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct CliArgs {
    /// Number of starting lines to skip translation
    #[arg(short, long, default_value = "0")]
    skip_lines: usize,
    /// Line content pattern to skip from translation
    ///
    /// Use this for any tags, comments, images etc that you want
    /// to keep as it is.
    #[arg(short, long, default_value = "^<.*>$")]
    pattern_skip: String,
    /// Append to the output file
    #[arg(short, long, action)]
    append: bool,
    /// Overwrite the output file
    #[arg(short, long, action)]
    overwrite: bool,
    /// Resume the previous Translation
    #[arg(short, long, action)]
    resume: bool,
    /// Input file in Japanese
    input_file: PathBuf,
    /// Output file in English
    output_file: PathBuf,
}

pub fn translate(args: CliArgs) -> Result<(), String> {
    let skip_rx = match Regex::new(&args.pattern_skip) {
        Ok(r) => r,
        Err(e) => return Err(format!("Invalid regex: {:?}", e)),
    };

    let file = match File::open(&args.input_file) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Couldn't open input file: {:?}\n{:?}",
                &args.input_file, e
            ))
        }
    };
    let mut reader_lines = BufReader::new(file).lines();

    let append = args.resume || args.append;
    let skip_lines = if args.resume {
        count_lines_in_file(&args.output_file)
    } else {
        args.skip_lines
    };

    let output_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .create_new(!(args.overwrite || append))
        .append(append)
        .truncate(!append)
        .open(args.output_file)
    {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::AlreadyExists => {
                return Err(concat!(
                    "Output File already exists, please use ",
                    "`--resume`, `--overwrite` or `--append` flag."
                )
                .to_string())
            }
            _ => return Err(error.to_string()),
        },
    };
    let mut writer = LineWriter::new(output_file);

    eprintln!("Building Translation model");
    let model = match TranslationModelBuilder::new()
        .with_model_type(ModelType::MBart)
        .with_source_languages(vec![Language::Japanese])
        .with_target_languages(vec![Language::English])
        .create_model()
    {
        Ok(m) => m,
        Err(e) => return Err(e.to_string()),
    };

    let pbar = get_progress_bar(args.input_file, skip_lines);
    let mut line: String;
    for _ in 0..skip_lines {
        reader_lines.next();
    }

    for (i, input_line) in reader_lines.enumerate() {
        line = input_line.unwrap();
        if line.trim() != "" {
            if skip_rx.is_match(&line) {
                match writer.write_all(line.as_bytes()) {
                    Ok(_) => (),
                    Err(e) => return Err(format!("{}\n{:?}", "Couldn't Write to Output File", e)),
                };
            } else {
                let output2 = model.translate(&[&line], Language::Japanese, Language::English);
                for sentence in output2 {
                    match writer.write_all(sentence.join(" ").as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(format!("{}\n{:?}", "Couldn't Write to Output File", e))
                        }
                    };
                }
            }
        }
        match writer.write_all(b"\n") {
            Ok(_) => (),
            Err(e) => return Err(format!("{}\n{:?}", "Couldn't Write to Output File", e)),
        };
        pbar.set_position((i + skip_lines + 1).try_into().unwrap());
    }
    Ok(())
}

fn get_progress_bar(input_file: PathBuf, skip_lines: usize) -> ProgressBar {
    let lines_len: u64 = count_lines_in_file(&input_file).try_into().unwrap();
    let pbar = ProgressBar::new(lines_len);
    let sty = ProgressStyle::default_bar()
        .template("{prefix:20} [{percent:>3.green}] {bar:50} {pos:>7}/{len:7} ETA: {eta} {msg}")
        .unwrap();
    pbar.set_style(sty);
    pbar.set_position(skip_lines.try_into().unwrap());
    pbar.set_prefix("Translating");
    return pbar;
}

fn count_lines_in_file(filename: &PathBuf) -> usize {
    let file = File::open(&filename).expect(&format!("Couldn't open file: {:?}", filename));
    let reader = BufReader::new(file);
    reader.lines().count()
}
