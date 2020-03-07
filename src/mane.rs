use std::{fs, io};
use std::borrow::Cow;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::Clap;
use rand::prelude::{IteratorRandom, SliceRandom};
use term_grid::{Direction, Filling, Grid, GridOptions};
use textwrap::{fill, wrap};
use unicode_width::UnicodeWidthStr;

use crate::pony::Pony;

mod pony;

const PREFIX: Option<&str> = option_env!("PREFIX");

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

    let prefix = PREFIX.unwrap_or(".");

    let pony_dir = &Path::new(prefix).join("share/ponysay/ponies");
    let pony_quote_dir = &Path::new(prefix).join("share/ponysay/quotes");

    if opts.list {
        print_pony_list(pony_dir)?;
    } else if opts.quote {
        print_pony_random_quote(pony_dir, pony_quote_dir)?;
    } else {
        // only check stdin if being piped to
        if atty::isnt(atty::Stream::Stdin) {
            let mut stdin_quote = String::new();
            io::stdin().read_to_string(&mut stdin_quote)?;
            print_random_pony(pony_dir, stdin_quote);
        }
    }

    Ok(())
}

fn print_random_pony(pony_dir: &Path, pony_quote: String) {
    let rng = &mut rand::thread_rng();

    let pony_paths: Vec<_> = paths(pony_dir).collect();
    let pony_path = pony_paths.choose(rng).unwrap();

    let mut pony = Pony::new(pony_path).expect(&format!("{} is not a pony file", pony_path.display()));

    println!("{}", pony.display(pony_quote));
}

fn print_pony_random_quote(pony_dir: &Path, pony_quote_dir: &Path) -> io::Result<()> {
    let rng = &mut rand::thread_rng();

    let pony_quote_paths: Vec<_> = paths(pony_quote_dir).collect();
    let pony_quote_path = pony_quote_paths.choose(rng).unwrap();

    let pony_names = pony_quote_path.file_stem().unwrap().to_string_lossy();
    let pony_name = pony_names
        .split("+")
        .choose(rng).unwrap();

    let mut pony = paths(pony_dir)
        .filter_map(|path| Pony::new(path))
        .find(|pony| pony.name() == pony_name)
        .expect(&format!("nopony is named {}", pony_name));

    let pony_quote = fs::read_to_string(&pony_quote_path)?;

    println!("{}", pony.display(pony_quote));

    Ok(())
}

fn paths(dir: &Path) -> impl Iterator<Item=PathBuf> {
    fs::read_dir(dir)
        .expect(&format!("unable to read {}", dir.display()))
        .into_iter()
        .filter_map(|entry| entry.ok().map(|e| e.path()))
}

fn print_pony_list(pony_dir: &Path) -> io::Result<()> {
    let mut pony_names = paths(pony_dir)
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

