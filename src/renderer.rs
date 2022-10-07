use clap::Args;
use maud::{html, Markup, DOCTYPE};
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Lines, Write};
use std::path::PathBuf;
use std::slice::IterMut;

#[derive(Args)]
pub struct CliArgs {
    /// Title of the Generated HTML
    #[arg(short, long, default_value = "Onegai | Combined Content")]
    title: String,
    /// Make a simple html without any contents check
    ///
    /// When your translations are not made with onegai tools, the
    /// lines might not match, this will help you generate simple
    /// html to see where the descripancy is at.
    #[arg(short, long, action)]
    simple_html: bool,
    /// Output file (html) to save combined page.
    #[arg(short, long, default_value = "onegai-combined.html")]
    output_file: PathBuf,
    /// Input files in different languages
    input_files: Vec<PathBuf>,
}

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

pub fn make_html(args: CliArgs) -> Result<(), String> {
    let num_lines = minimum_common_line(&args.input_files, !args.simple_html)?;

    let mut readers: Vec<Lines<BufReader<File>>> = args
        .input_files
        .iter()
        .map(|fp| {
            let file = File::open(&fp).unwrap();
            let lines = BufReader::new(file).lines();
            return lines;
        })
        .collect();

    let out_file = match File::create(&args.output_file) {
        Ok(f) => f,
        Err(e) => return Err(format!("{}\n{:?}", "Cannot create output file", e)),
    };
    let mut writer = LineWriter::new(out_file);
    if args.simple_html {
        match writer.write(
            get_simple_html(args.title, num_lines, readers)
                .into_string()
                .as_bytes(),
        ) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(format!("{}\n{:?}", "Cannot write to output file", e)),
        };
    }

    let contents = get_file_contents(readers.iter_mut(), num_lines);
    match writer.write(
        get_chapter_html(args.title, contents)
            .into_string()
            .as_bytes(),
    ) {
        Ok(_) => (),
        Err(e) => return Err(format!("{}\n{:?}", "Cannot write to output file", e)),
    };
    println!("Output saved: {:?}", args.output_file);
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
            div.chap-line id=(linetag) {
        div {
            @for reader in readers.iter_mut() {
            @match reader.next() {
                Some(line) => div.chap-line-contents {(line.unwrap())},
                None => div.chap-line-contents {""},
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
            div.chap-line id=(linetag) {
        @match line.linetype {
            LineType::Contents => div.chap-line-contents {
            @for line in line.contents.iter() {
                (line)
                br;
            }
            },
            LineType::ParagraphBreak => p.para-break {}
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

fn minimum_common_line(input_files: &Vec<PathBuf>, check_len: bool) -> Result<usize, String> {
    if input_files.len() == 0 {
        return Err(format!("Empty File list: {:?}", input_files));
    }
    let mut lines = Vec::<usize>::with_capacity(input_files.len());
    let mut min_lines: usize = usize::max_value();
    for (i, f) in input_files.iter().enumerate() {
        match File::open(&f) {
            Ok(fil) => lines.push(BufReader::new(fil).lines().count()),
            Err(e) => {
                return Err(format!(
                    "Couldn't open input file {}.\nError: {:?}",
                    f.as_os_str().to_str().unwrap(),
                    e
                ));
            }
        };
        if lines[i] < min_lines {
            min_lines = lines[i];
        }
    }
    if check_len && !lines.iter().all(|v| *v == lines[0]) {
        return Err(format!("{}: {:?}", "File lengths don't match", lines));
    }
    return Ok(min_lines);
}
