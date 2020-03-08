use std::{fs, io};
use std::fs::read_to_string;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::Clap;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::Rng;
use term_grid::{Direction, Filling, Grid, GridOptions};
use textwrap::fill;

use crate::pony::Pony;

mod pony;

const PREFIX: Option<&str> = option_env!("PREFIX");

#[derive(Clap)]
#[clap(version = "1.0", author = "Pixel Light")]
struct Opts {
    /// List pony names.
    #[clap(short, long, exclusive = true)]
    list: bool,
    /// Pick a random quote.
    #[clap(short, long, exclusive = true)]
    quote: bool,
    /// Pick a specific name or file.
    #[clap(short, long, alias = "pony", multiple = true)]
    file: Vec<String>,
    #[clap(name = "QUOTE")]
    input: Option<String>,
}

enum Constraint {
    Path(PathBuf),
    Name(String),
}

fn main() {
    let opts = Opts::parse();

    let prefix = PREFIX.unwrap_or(".");

    let pony_dir = &Path::new(prefix).join("share/ponysay/ponies");
    let pony_quote_dir = &Path::new(prefix).join("share/ponysay/quotes");

    if opts.list {
        print_pony_list(pony_dir);
    } else {
        if let Some((pony_quote, constraint)) = calculate_quote(&opts, pony_quote_dir) {
            let mut constraints = calculate_constraints(&opts);
            if let Some(quote_constraint) = constraint {
                constraints.push(quote_constraint);
            }
            let possible_ponies = constrain_ponies(pony_dir, constraints);
            let mut pony = select_pony(possible_ponies);

            println!("{}", pony.display(pony_quote));
        }
    }
}

fn calculate_quote(opts: &Opts, pony_quote_dir: &Path) -> Option<(String, Option<Constraint>)> {
    if opts.quote {
        let rng = &mut rand::thread_rng();

        let pony_quote_paths: Vec<_> = paths(pony_quote_dir).collect();
        let pony_quote_path = pony_quote_paths.choose(rng).unwrap();

        let pony_names = pony_quote_path.file_stem().unwrap().to_string_lossy();
        let pony_name = pony_names
            .split("+")
            .choose(rng).unwrap().to_string();

        let pony_quote = fs::read_to_string(pony_quote_path).expect(&format!("unable to read {}", pony_quote_path.display()));
        Some((pony_quote, Some(Constraint::Name(pony_name))))
    } else {
        println!("input: {:?}", opts.input);
        if let Some(pony_quote) = &opts.input {
            Some((pony_quote.to_string(), None))
        } else if atty::isnt(atty::Stream::Stdin) {
            // only check stdin if being piped to
            let mut stdin_quote = String::new();
            io::stdin().read_to_string(&mut stdin_quote).unwrap();
            Some((stdin_quote, None))
        } else {
            None
        }
    }
}

fn calculate_constraints(opts: &Opts) -> Vec<Constraint> {
    opts.file.iter().map(|file_or_name| {
        if file_or_name.ends_with(".pony") {
            // assume file
            Constraint::Path(Path::new(file_or_name).to_path_buf())
        } else {
            Constraint::Name(file_or_name.clone())
        }
    }).collect()
}

fn select_pony(ponies: Vec<Pony>) -> Pony {
    if ponies.is_empty() {
        eprintln!("Couldn't find anypony");
        exit(1)
    } else if ponies.len() == 1 {
        ponies.into_iter().nth(0).unwrap()
    } else {
        let rng = &mut rand::thread_rng();
        let index = rng.gen_range(0, ponies.len());
        ponies.into_iter().nth(index).unwrap()
    }
}

fn constrain_ponies(pony_dir: &Path, constraints: Vec<Constraint>) -> Vec<Pony> {
    let paths = paths(pony_dir)
        .filter_map(|path| Pony::new(path));
    if constraints.is_empty() {
        paths.collect()
    } else {
        let mut result = vec![];
        for constraint in &constraints {
            match constraint {
                Constraint::Path(path) => {
                    if let Some(pony) = Pony::new(path) {
                        result.push(pony)
                    }
                }
                _ => {}
            }
        }
        result.extend(paths.filter(|pony| constraints.iter().any(|constraint| matches(pony, constraint))));

        result
    }
}

fn matches(pony: &Pony, constraint: &Constraint) -> bool {
    match constraint {
        Constraint::Name(name) => { pony.name() == name }
        _ => false
    }
}

fn paths(dir: &Path) -> impl Iterator<Item=PathBuf> {
    fs::read_dir(dir)
        .expect(&format!("unable to read {}", dir.display()))
        .into_iter()
        .filter_map(|entry| entry.ok().map(|e| e.path()))
}

fn print_pony_list(pony_dir: &Path) {
    let mut ponies = paths(pony_dir)
        .filter_map(|path| Pony::new(&path))
        .collect::<Vec<_>>();
    ponies.sort();

    if let Some((term_width, _)) = term_size::dimensions_stdout() {
        println!("ponies located in {}", pony_dir.display());
        print_columns(ponies, term_width);
    } else {
        print_single_column(ponies);
    }
}

fn print_columns(ponies: Vec<Pony>, size: usize) {
    let mut grid = Grid::new(GridOptions {
        filling: Filling::Spaces(2),
        direction: Direction::TopToBottom,
    });

    for pony in ponies {
        grid.add(pony.name().into());
    }

    if let Some(output) = grid.fit_into_width(size) {
        println!("{}", output);
    } else {
        println!("{}", grid.fit_into_columns(1));
    }
}

fn print_single_column(ponies: Vec<Pony>) {
    for pony in ponies {
        println!("{}", pony.name());
    }
}
