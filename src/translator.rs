use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn translate(input_file: PathBuf, _output_file: PathBuf) {
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
