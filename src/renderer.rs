use maud::{html, Markup, DOCTYPE};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::PathBuf;
use std::slice::IterMut;

struct ChapterLine {
    linetype: LineType,
    linenum: usize,
    contents: Vec<String>,
}

enum LineType {
    Contents,
    ParagraphBreak,
}

impl ChapterLine {
    fn new(linum: usize, contents: String) -> ChapterLine {
        if contents.trim() == "" {
            return ChapterLine {
                linetype: LineType::ParagraphBreak,
                linenum: linum,
                contents: vec![],
            };
        } else {
            return ChapterLine {
                linetype: LineType::Contents,
                linenum: linum,
                contents: vec![contents],
            };
        }
    }
    fn add(&mut self, contents: String) {
        match self.linetype {
            LineType::Contents => self.contents.push(contents),
            LineType::ParagraphBreak => {
                if contents.trim() != "" {
                    eprint!("{}: {}", self.linenum, contents);
                    panic!("Contents in Paragraph Break.")
                }
            }
        }
    }
}

pub fn make_html(title: String, ignore_match_err: bool, input_files: Vec<PathBuf>) {
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
    let num_lines: usize = *lines.iter().min().unwrap();

    let mut readers: Vec<Lines<BufReader<File>>> = input_files
        .iter()
        .map(|fp| {
            let file = File::open(fp.clone()).unwrap();
            let lines = BufReader::new(file).lines();
            return lines;
        })
        .collect();

    if !lines.iter().all(|v| *v == lines[0]) {
        if !ignore_match_err {
            panic!("File lengths doesn't match. {:?}", lines);
        }
        println!(
            "{}",
            get_simple_html(title, num_lines, readers).into_string()
        );
        return;
    }
    let contents = get_file_contents(readers.iter_mut(), num_lines);
    println!("{}", get_chapter_html(title, contents).into_string());
}

fn get_simple_html(
    title: String,
    num_lines: usize,
    mut readers: Vec<Lines<BufReader<File>>>,
) -> Markup {
    html! {
    (DOCTYPE)
        title {(title)}
        @for i in 1..=num_lines {
            @let linetag=format!("line-{}",i);
            div id=(linetag) {
        div {
            @for reader in readers.iter_mut() {
            @match reader.next() {
                Some(line) => div {(line.unwrap())},
                None => div {""},
            }
        br;
            }
        }}
        }}
}

fn get_chapter_html(title: String, chap_lines: Vec<ChapterLine>) -> Markup {
    html! {
    (DOCTYPE)
        title {(title)}
        @for line in chap_lines.iter() {
            @let linetag=format!("line-{}", line.linenum);
            div id=(linetag) {
        @match line.linetype {
            LineType::Contents => div {
            @for line in line.contents.iter() {
                (line)
                br;
            }
            },
            LineType::ParagraphBreak => p {}
            }
        }}
    }
}

fn get_file_contents(
    mut readers: IterMut<Lines<BufReader<File>>>,
    num_lines: usize,
) -> Vec<ChapterLine> {
    let mut lines = Vec::<ChapterLine>::with_capacity(num_lines);
    let reader = readers.next().unwrap();
    for i in 0..num_lines {
        match reader.next() {
            Some(line) => lines.push(ChapterLine::new(i, line.unwrap())),
            None => break,
        }
    }
    for reader in readers {
        for i in 0..num_lines {
            match reader.next() {
                Some(line) => lines[i].add(line.unwrap()),
                None => break,
            }
        }
    }
    return lines;
}
