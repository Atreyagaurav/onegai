use indicatif::{ProgressBar, ProgressStyle};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, LineWriter, Write};
use std::path::PathBuf;

pub fn translate(
    skip_lines: usize,
    append: bool,
    overwrite: bool,
    input_file: PathBuf,
    output_file: PathBuf,
) -> Result<(), String> {
    let file = match File::open(&input_file) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Couldn't open input file: {:?}\n{:?}",
                &input_file, e
            ))
        }
    };
    let mut reader_lines = BufReader::new(file).lines();

    let output_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .create_new(!(overwrite || append))
        .append(append)
        .truncate(!append)
        .open(output_file)
    {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::AlreadyExists => {
                return Err(concat!(
                    "Output File already exists, please use ",
                    "`--overwrite` or `--append` flag."
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

    let pbar = get_progress_bar(input_file, skip_lines);
    let mut line: String;
    for _ in 0..skip_lines {
        reader_lines.next();
    }

    for (i, input_line) in reader_lines.enumerate() {
        line = input_line.unwrap();
        if line.trim() != "" {
            let output2 = model.translate(&[line.clone()], Language::Japanese, Language::English);
            for sentence in output2 {
                match writer.write_all(sentence.join(" ").as_bytes()) {
                    Ok(_) => (),
                    Err(e) => return Err(format!("{}\n{:?}", "Couldn't Write to Output File", e)),
                };
            }
        }
        match writer.write_all(b"\n") {
            Ok(_) => (),
            Err(e) => return Err(format!("{}\n{:?}", "Couldn't Write to Output File", e)),
        };
        pbar.set_position(i.try_into().unwrap());
    }
    Ok(())
}

fn get_progress_bar(input_file: PathBuf, skip_lines: usize) -> ProgressBar {
    let file = File::open(&input_file).expect("Couldn't open input file.");
    let reader = BufReader::new(file);
    let lines_len: u64 = reader.lines().count().try_into().unwrap();
    let pbar = ProgressBar::new(lines_len);

    let sty = ProgressStyle::default_bar()
        .template("{prefix:20} [{percent:>3.green}] {bar:50} {pos:>7}/{len:7} ETA: {eta} {msg}")
        .unwrap();
    pbar.set_style(sty);
    pbar.set_position(skip_lines.try_into().unwrap());
    pbar.set_prefix("Translating");
    return pbar;
}
