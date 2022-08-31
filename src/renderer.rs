use maud::{html, Markup, DOCTYPE};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::PathBuf;

pub fn make_html(ignore_match_err: bool, input_files: Vec<PathBuf>) {
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
