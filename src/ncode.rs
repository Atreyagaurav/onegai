use regex::Regex;
use reqwest;
use reqwest::header::USER_AGENT;
use select::{document::Document, node::Node, predicate::Attr};
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;

// url example: https://ncode.syosetu.com/n3286gu/1/

struct Ncode {
    novel: String,
    chapter: String,
}

impl Ncode {
    fn url(self) -> String {
        return format!("https://ncode.syosetu.com/{}/{}", self.novel, self.chapter);
    }

    fn download(self) -> String {
        let client = reqwest::blocking::Client::new();
        let body = client
            .get(self.url())
            .header(USER_AGENT, "My Rust Program 1.0")
            .send()
            .expect("HTTP Req failed.");
        let parsed = Document::from_read(body).unwrap();
        let node: Node = parsed
            .find(Attr("id", "novel_honbun"))
            .next()
            .expect("Ncode novel contents not found.");
        return node.text();
    }

    fn save(self, outfile: PathBuf) {
        let output_file = File::create(outfile).expect("Couldn't create file.");
        let mut writer = LineWriter::new(output_file);
        writer.write(&self.download().into_bytes()).unwrap();
    }
}

pub fn download_ncode(address: String, outfile: PathBuf) {
    let ncode_re =
        Regex::new(r"^(https?://)?(ncode.syosetu.com/)?([a-z0-9]+)/([a-z0-9]+)/?$").unwrap();
    let caps = ncode_re
        .captures(&address)
        .expect("Provided Url doesn't look like from syosetsu.com");
    let chapter = Ncode {
        novel: caps.get(3).unwrap().as_str().to_string(),
        chapter: caps.get(4).unwrap().as_str().to_string(),
    };
    chapter.save(outfile);
}
