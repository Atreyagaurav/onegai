use clap::Args;
use colored::Colorize;
use regex::Regex;
use reqwest;
use reqwest::header::USER_AGENT;
use select::{document::Document, node::Node, predicate::Attr};
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct CliArgs {
    /// Chapter url
    ///
    /// Chapter url should be from syosetu.com. Examples of
    /// supported urls are:
    /// https://ncode.syosetu.com/n2267be/561/,
    /// ncode.syosetu.com/n2267be/561/, n2267be/561/, etc
    ncode_url: String,
    /// Output file to save the chapter
    output_file: PathBuf,
}

// url example: https://ncode.syosetu.com/n3286gu/1/

struct Ncode {
    novel: String,
    chapter: String,
}

impl Ncode {
    fn url(&self) -> String {
        return format!("https://ncode.syosetu.com/{}/{}", self.novel, self.chapter);
    }

    fn download(&self) -> Result<String, String> {
        let client = reqwest::blocking::Client::new();
        let body = match client
            .get(self.url())
            .header(USER_AGENT, "My Rust Program 1.0")
            .send()
        {
            Ok(b) => b,
            Err(e) => return Err(format!("{}\n{:?}", "HTTP Req failed.", e)),
        };
        let parsed = Document::from_read(body).unwrap();
        let node: Node = match parsed.find(Attr("id", "novel_honbun")).next() {
            Some(n) => n,
            None => return Err(format!("{}", "Novel contents not found.")),
        };
        return Ok(node.text());
    }

    fn save(&self, outfile: PathBuf) -> Result<(), String> {
        let output_file = File::create(outfile).expect("Couldn't create file.");
        let mut writer = LineWriter::new(output_file);
        match writer.write(&format!("{}\n", (self.download()?).trim()).into_bytes()) {
            Ok(_) => (),
            Err(e) => return Err(format!("{}\n{:?}", "Cannot write to output file", e)),
        }
        Ok(())
    }
}

pub fn download_ncode(args: CliArgs) -> Result<(), String> {
    let ncode_re =
        Regex::new(r"^(https?://)?(ncode.syosetu.com/)?([a-z0-9]+)/([a-z0-9]+)/?$").unwrap();
    let caps = match ncode_re.captures(&args.ncode_url) {
        Some(m) => m,
        None => {
            return Err(format!(
                "{}\n{}",
                "Provided Url doesn't look like from syosetsu.com",
                "Use `help download' to get more details on valid urls"
            ))
        }
    };
    let chapter = Ncode {
        novel: caps.get(3).unwrap().as_str().to_string(),
        chapter: caps.get(4).unwrap().as_str().to_string(),
    };

    println!("{}: {}", "Requesting".green().bold(), chapter.url());
    chapter.save(args.output_file)?;
    Ok(())
}
