use std::{fs, io};
use std::path::Path;

use clap::Clap;
use std::ffi::OsStr;
use term_grid::{Grid, GridOptions, Filling, Direction};

#[derive(Clap)]
#[clap(version = "1.0", author = "Pixel Light")]
struct Opts {
    /// List pony names.
    #[clap(short, long, exclusive = true)]
    list: bool,
}

fn main() -> io::Result<()> {
    let opts = Opts::parse();

    if opts.list {
        let pony_dir = Path::new("/usr/share/ponysay/ponies/");
        print_pony_list(pony_dir)?;
    } else {}

    Ok(())
}

fn print_pony_list(pony_dir: &Path) -> io::Result<()> {
    let mut pony_names = fs::read_dir(pony_dir)?.into_iter()
        .filter_map(|entry| entry.ok().and_then(|e| to_pony_name(&e.path())))
        .collect::<Vec<_>>();
    pony_names.sort();

    if let Some((term_width, _)) = term_size::dimensions() {
        print_columns(pony_names, term_width);
    } else {
        print_single_column(pony_names);
    }

    Ok(())
}

fn print_columns(pony_names: Vec<String>, size: usize) {
    let mut grid = Grid::new(GridOptions {
        filling:     Filling::Spaces(2),
        direction:   Direction::TopToBottom,
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
