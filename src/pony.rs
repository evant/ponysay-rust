use std::cmp::Ord;
use std::convert::Into;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag};
use nom::combinator::{map, opt};
use nom::lib::std::cmp::Ordering;
use nom::multi::many0;
use nom::sequence::{delimited, pair, separated_pair, terminated};
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct Pony {
    path: PathBuf,
    name: String,
    pony: Option<String>,
}

impl Pony {
    pub fn new<P: Into<PathBuf>>(path: P) -> Option<Pony> {
        let path = path.into();
        let name = if path.extension() == Some(OsStr::new("pony")) {
            Some(path.file_stem()?.to_str()?.to_owned())
        } else {
            None
        };
        name.map(|name| {
            Pony { path, name, pony: None }
        })
    }

    pub fn path(&self) -> &Path { &self.path }

    pub fn name(&self) -> &str { &self.name }

    pub fn display(&mut self, pony_quote: String) -> String {
        let pony = self.pony();
        let (_, (_metadata, pony)) = parse_pony(pony, create_balloon(&pony_quote)).unwrap();
        return pony;
    }

    fn pony(&mut self) -> &str {
        let path = &self.path;
        self.pony.get_or_insert_with(|| fs::read_to_string(path).expect(&format!("unable read pony {}", path.display())))
    }
}

fn parse_pony(pony: &str, quote: String) -> nom::IResult<&str, (Vec<(&str, &str)>, String)> {
    let (pony, metadata) = parse_metadata(pony)?;
    let (pony, body) = parse_pony_body(pony, quote)?;
    Ok((pony, (metadata, body)))
}

fn parse_balloon(pony: &str, quote: String) -> nom::IResult<&str, String> {
    let (pony, _) = delimited(tag("$"), is_not("$"), tag("$"))(pony)?;
    Ok((pony, quote))
}

fn parse_metadata(pony: &str) -> nom::IResult<&str, Vec<(&str, &str)>> {
    delimited(tag("$$$\n"), many0(parse_metadata_line), pair(is_not("$"), tag("$$$\n")))(pony)
}

fn parse_metadata_line(pony: &str) -> nom::IResult<&str, (&str, &str)> {
    terminated(separated_pair(is_not(":\n"), tag(":"), is_not("\n")), tag("\n"))(pony)
}

fn parse_pony_body(pony: &str, quote: String) -> nom::IResult<&str, String> {
    let (pony, start) = parse_pony_body2(pony)?;
    let (pony, balloon) = parse_balloon(pony, quote)?;
    let (pony, end) = parse_pony_body2(pony)?;
    Ok((pony, start + &balloon + &end))
}

fn parse_pony_body2(pony: &str) -> nom::IResult<&str, String> {
    map(many0(alt((
        map(parse_stem, |s| s.into()),
        map(is_not("$"), |s: &str| s.into())
    ))), |v: Vec<String>| v.join(""))(pony)
}

fn parse_stem(pony: &str) -> nom::IResult<&str, String> {
    map(delimited(tag("$"), is_a("\\/"), tag("$")), |stem| format!(" {} ", stem))(pony)
}

fn create_balloon(quote: &str) -> String {
    let wrapped_text = wrap(quote, 65);
    let width = wrapped_text.iter()
        .map(|t| t.width())
        .fold(0, usize::max);

    let mut output = String::new();
    output.push(' ');
    for _ in 0..(width + 2) {
        output.push('_');
    }
    output.push(' ');
    output.push('\n');
    if wrapped_text.len() == 1 {
        output.push_str(&format!("< {} >\n", wrapped_text[0].trim_end()));
    } else {
        for i in 0..wrapped_text.len() {
            let line = wrapped_text[i].trim_end();
            let end_padding = width - line.width();
            let (start_char, end_char) = if i == 0 {
                ('/', '\\')
            } else if i == wrapped_text.len() - 1 {
                ('\\', '/')
            } else {
                ('|', '|')
            };
            output.push(start_char);
            output.push(' ');
            output.push_str(line);
            for _ in 0..(end_padding + 1) {
                output.push(' ');
            }
            output.push(end_char);
            output.push('\n');
        }
    }
    output.push(' ');
    for _ in 0..(width + 2) {
        output.push('-');
    }
    output.push(' ');
    return output;
}

impl Ord for Pony {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Pony {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Eq for Pony {}

impl PartialEq for Pony {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::Path;

    use crate::pony::Pony;

    #[test]
    fn parse_all_ponies() {
        let pony_dir = Path::new("share/ponysay/ponies");
        let ponies = fs::read_dir(pony_dir)
            .expect(&format!("unable to read {}", pony_dir.display()))
            .into_iter()
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .map(|path| Pony::new(path).unwrap());

        for mut pony in ponies {
            pony.display("Hi".to_string());
        }
    }
}
