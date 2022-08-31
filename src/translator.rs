use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};


pub fn translate(skip_lines: usize, append: bool, input_file: PathBuf, output_file: PathBuf) {
    let file = File::open(&input_file).expect("Couldn't open input file.");
    let mut reader_lines = BufReader::new(file).lines();

    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(append)
        .truncate(!append)
        .open(output_file)
        .unwrap();
    let mut writer = LineWriter::new(output_file);

    let model = TranslationModelBuilder::new()
        .with_model_type(ModelType::MBart)
        .with_source_languages(vec![Language::Japanese])
        .with_target_languages(vec![Language::English])
        .create_model()
        .unwrap();

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
                writer
                    .write_all(sentence.join(" ").as_bytes())
                    .expect("Couldn't Write to Output File.");
            }
        }
        writer
            .write_all(b"\n")
            .expect("Couldn't Write to Output File.");
	pbar.set_position(i.try_into().unwrap());
    }
}


fn get_progress_bar(input_file:PathBuf, skip_lines:usize) -> ProgressBar{
    let file = File::open(&input_file).expect("Couldn't open input file.");
    let reader = BufReader::new(file);
    let lines_len:u64 = reader.lines().count().try_into().unwrap();
    let pbar = ProgressBar::new(lines_len);

    let sty = ProgressStyle::default_bar().template("{prefix:20} [{percent:>3.green}] {bar:50} {pos:>7}/{len:7} {eta} {msg}"
    ).unwrap();
    pbar.set_style(sty);
    pbar.set_position(skip_lines.try_into().unwrap());
    pbar.set_prefix("Translating");
    return pbar;
}
