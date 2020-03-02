use std::{fs, io};
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use clap::Clap;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, take_until, take_while};
use nom::character::complete::anychar;
use nom::combinator::{all_consuming, map, not, recognize};
use nom::error::ParseError;
use nom::InputLength;
use nom::multi::many0;
use nom::sequence::{delimited, pair, separated_pair, terminated, tuple};
use rand::prelude::{IteratorRandom, SliceRandom};
use term_grid::{Direction, Filling, Grid, GridOptions};
use textwrap::{fill, wrap};
use unicode_width::UnicodeWidthStr;
use std::io::Read;

#[derive(Clap)]
#[clap(version = "1.0", author = "Pixel Light")]
struct Opts {
    /// List pony names.
    #[clap(short, long, exclusive = true)]
    list: bool,
    #[clap(short, long, exclusive = true)]
    quote: bool,
}

fn main() -> io::Result<()> {
    let opts = Opts::parse();

    let pony_dir = Path::new("/usr/local/share/ponysay/ponies/");
    let pony_quote_dir = Path::new("/usr/local/share/ponysay/quotes");


    if opts.list {
        print_pony_list(pony_dir)?;
    } else if opts.quote {
        print_pony_random_quote(pony_dir, pony_quote_dir)?;
    } else {
        // only check stdin if being piped to
        if atty::isnt(atty::Stream::Stdin) {
            let mut stdin_quote = String::new();
            io::stdin().read_to_string(&mut stdin_quote)?;
            print_random_pony(pony_dir, stdin_quote)?;
        }
    }

    Ok(())
}

fn print_random_pony(pony_dir: &Path, pony_quote: String) -> io::Result<()> {
    let rng = &mut rand::thread_rng();

    let pony_paths: Vec<_> = paths(pony_dir)?.collect();
    let pony_path = pony_paths.choose(rng).unwrap();

    let pony = fs::read_to_string(pony_path)?;

    print_pony(pony, pony_quote);

    Ok(())
}

fn print_pony_random_quote(pony_dir: &Path, pony_quote_dir: &Path) -> io::Result<()> {
    let rng = &mut rand::thread_rng();

    let pony_quote_paths: Vec<_> = paths(pony_quote_dir)?.collect();
    let pony_quote_path = pony_quote_paths.choose(rng).unwrap();

    let pony_names = pony_quote_path.file_stem().unwrap().to_string_lossy();
    let pony_name = pony_names
        .split("+")
        .choose(rng).unwrap();

    let pony_path = paths(pony_dir)?
        .find(|path| to_pony_name(path).map(|n| n == pony_name).unwrap_or(false))
        .unwrap();

    let pony_quote = fs::read_to_string(&pony_quote_path)?;
    let pony = fs::read_to_string(pony_path)?;

    print_pony(pony, pony_quote);

    Ok(())
}

fn print_pony(pony: String, pony_quote: String) {
    let (_, (_metadata, pony)) = parse_pony(create_balloon(&pony_quote), &pony).unwrap();

    println!("{}", pony);
}

fn parse_pony(quote: String, pony: &str) -> nom::IResult<&str, (Vec<(&str, &str)>, String)> {
    pair(parse_metadata, parse_pony_body(quote))(pony)
}

fn parse_balloon<'a>(quote: String) -> impl Fn(&'a str) -> nom::IResult<&'a str, String> {
    move |pony| {
        let (pony, _) = delimited(tag("$"), is_not("$"), tag("$"))(pony)?;
        Ok((pony, quote.clone()))
    }
}

fn parse_metadata(pony: &str) -> nom::IResult<&str, Vec<(&str, &str)>> {
    delimited(tag("$$$\n"), many0(parse_metadata_line), pair(is_not("$"), tag("$$$\n")))(pony)
}

fn parse_metadata_line(pony: &str) -> nom::IResult<&str, (&str, &str)> {
    terminated(separated_pair(is_not(":"), tag(":"), is_not("\n")), tag("\n"))(pony)
}

fn parse_pony_body<'a>(quote: String) -> impl Fn(&'a str) -> nom::IResult<&'a str, String> {
    move |pony| {
        let (pony, v): (_, Vec<Cow<str>>) = many0(alt((
            map(parse_stem, |s| s.into()),
            map(parse_balloon(quote.clone()), |s| s.into()),
            map(is_not("$"), |s: &str| s.into())
        )))(pony)?;
        Ok((pony, v.join("")))
    }
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

fn paths(dir: &Path) -> io::Result<impl Iterator<Item=PathBuf>> {
    Ok(fs::read_dir(dir)?.into_iter()
        .filter_map(|entry| entry.ok().map(|e| e.path())))
}

fn print_pony_list(pony_dir: &Path) -> io::Result<()> {
    let mut pony_names = paths(pony_dir)?
        .filter_map(|path| to_pony_name(&path))
        .collect::<Vec<_>>();
    pony_names.sort();

    if let Some((term_width, _)) = term_size::dimensions_stdout() {
        println!("ponies located in {}", pony_dir.display());
        print_columns(pony_names, term_width);
    } else {
        print_single_column(pony_names);
    }

    Ok(())
}

fn print_columns(pony_names: Vec<String>, size: usize) {
    let mut grid = Grid::new(GridOptions {
        filling: Filling::Spaces(2),
        direction: Direction::TopToBottom,
    });

    for pony_name in pony_names {
        grid.add(pony_name.into());
    }

    if let Some(output) = grid.fit_into_width(size) {
        println!("{}", output);
    } else {
        println!("{}", grid.fit_into_columns(1));
    }
}

fn print_single_column(pony_names: Vec<String>) {
    for pony_name in pony_names {
        println!("{}", pony_name);
    }
}

fn to_pony_name(path: &Path) -> Option<String> {
    if path.extension() == Some(OsStr::new("pony")) {
        Some(path.file_stem()?.to_str()?.to_owned())
    } else {
        None
    }
}

pub fn eof<I: Copy + InputLength, E: ParseError<I>>(input: I) -> nom::IResult<I, I, E> {
    if input.input_len() == 0 {
        Ok((input, input))
    } else {
        Err(nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::Eof)))
    }
}
