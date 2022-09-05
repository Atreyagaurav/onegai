use maud::{html, Markup, DOCTYPE};
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Lines, Write};
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
                    eprint!("Line {}: {}\n", self.linenum, contents);
                    panic!("Contents in Paragraph Break.")
                }
            }
        }
    }
}

pub fn make_html(
    title: String,
    simple_html: bool,
    output_file: PathBuf,
    input_files: Vec<PathBuf>,
) -> Result<(), String> {
    let lines: Vec<usize> = input_files
        .iter()
        .map(|f| {
            let file = File::open(&f).expect(&format!(
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
            let file = File::open(&fp).unwrap();
            let lines = BufReader::new(file).lines();
            return lines;
        })
        .collect();

    let out_file = match File::create(&output_file) {
        Ok(f) => f,
        Err(e) => return Err(format!("{}\n{:?}", "Cannot create output file", e)),
    };
    let mut writer = LineWriter::new(out_file);
    if simple_html {
        match writer.write(
            get_simple_html(title, num_lines, readers)
                .into_string()
                .as_bytes(),
        ) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(format!("{}\n{:?}", "Cannot write to output file", e)),
        };
    }

    if !lines.iter().all(|v| *v == lines[0]) {
        return Err(format!("{}: {:?}", "File lengths doesn't match", lines));
    }

    let contents = get_file_contents(readers.iter_mut(), num_lines);
    match writer.write(get_chapter_html(title, contents).into_string().as_bytes()) {
        Ok(_) => (),
        Err(e) => return Err(format!("{}\n{:?}", "Cannot write to output file", e)),
    };
    println!("Output saved: {:?}", output_file);
    Ok(())
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
