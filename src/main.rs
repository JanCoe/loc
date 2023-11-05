use clap::Parser;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::fmt;

#[derive(Parser)]
struct Cli {
    path: PathBuf,
    extension: String,
}

#[derive(Debug)]
struct Count {
    code: u32,
    comment: u32,
    blank: u32,
}

impl Count {
    fn new() -> Self {
        Self {
            code: 0,
            comment: 0,
            blank: 0,
        }
    }
}

impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Code: {code:>4}; Comments: {comments:>4}; Blank: {blank:>4}",
            code = self.code,
            comments = self.comment,
            blank = self.blank)
    }
}
enum LineType {
    Blank,
    Code,
    Comment,
    CommentMulti,
}

fn count_lines(path: &str, total_count: &mut Count) {
    let content = std::fs::read_to_string(path).expect("could not read file");

    let re_hashtag = Regex::new(r"^\s*#").unwrap();
    let re_triple_quote_start = Regex::new(r#"^\s*""""#).unwrap();
    let re_triple_quote_end = Regex::new(r#""""$"#).unwrap();
    let re_triple_quote_twice = Regex::new(r#"^\s*""".*"""$"#).unwrap();

    let mut local_count = Count::new();
    let mut line_type = LineType::Blank;

    for line in content.lines() {
        match line_type {
            LineType::CommentMulti => {
                if re_triple_quote_end.is_match(line) {
                    line_type = LineType::Comment
                } else {
                    line_type = LineType::CommentMulti
                }
            }
            _ => {
                if line.is_empty() {
                    line_type = LineType::Blank
                } else if re_hashtag.is_match(line) | re_triple_quote_twice.is_match(line) {
                    line_type = LineType::Comment
                } else if re_triple_quote_start.is_match(line) {
                    line_type = LineType::CommentMulti
                } else {
                    line_type = LineType::Code
                }
            }
        };

        match line_type {
            LineType::Blank => local_count.blank += 1,
            LineType::Code => local_count.code += 1,
            _ => local_count.comment += 1,
        }
    }

    println!("{local_count}; Path: {path}");

    total_count.code += local_count.code;
    total_count.comment += local_count.comment;
    total_count.blank += local_count.blank;
}

fn has_extension(path: &Path, extension: &str) -> bool {
    if let Some(file_extension) = path.extension() {
        if file_extension == extension {
            return true;
        }
    }
    false
}

fn visit_dirs(path: &str, extension: &str, total_count: &mut Count) -> std::io::Result<()> {
    let entries = std::fs::read_dir(path).expect("could not read dir");

    for entry in entries {
        let entry = entry?;
        let path_of_entry = entry.path();

        if path_of_entry.is_file() & has_extension(&path_of_entry, extension) {
            count_lines(&path_of_entry.to_string_lossy(), total_count);
        } else if path_of_entry.is_dir() {
            visit_dirs(&path_of_entry.to_string_lossy(), extension, total_count)?;
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    let mut total_count = Count::new();

    visit_dirs(
        &args.path.to_string_lossy(),
        &args.extension,
        &mut total_count,
    )?;

    println!("{total_count}; Path: {path}", path=&args.path.to_string_lossy());

    Ok(())
}
