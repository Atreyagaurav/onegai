use clap::Args;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tts_rust::{languages::Languages, GTTSClient};

#[derive(Args)]
pub struct CliArgs {
    /// the input file is in Japanese not English
    #[arg(short, long, action)]
    is_jp: bool,
    /// Input file path in English or Japanese
    input_file: PathBuf,
}

pub fn gspeak(args: CliArgs) -> Result<(), String> {
    let narrator: GTTSClient = GTTSClient {
        volume: 1.0,
        language: if args.is_jp {
            Languages::Japanese
        } else {
            Languages::English
        }, // use the Languages enum
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
    let reader_lines = BufReader::new(file).lines();
    for line in reader_lines {
        match line {
            Ok(ls) => {
                if !ls.trim().is_empty() {
                    println!("{}", &ls);
                    narrator.speak(&ls);
                } else {
                    println!("");
                }
            }
            Err(e) => return Err(format!("Invalid line: {:?}", e)),
        }
    }
    Ok(())
}
