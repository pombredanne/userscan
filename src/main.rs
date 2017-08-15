extern crate atty;
extern crate bytesize;
extern crate colored;
extern crate env_logger;
extern crate flate2;
extern crate ignore;
extern crate nix;
extern crate serde;
extern crate serde_json;
extern crate tree_magic;
extern crate users;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate tempdir;

mod cache;
mod errors;
mod output;
mod registry;
mod scan;
mod statistics;
mod walk;
#[cfg(test)]
mod tests;

use cache::Cache;
use bytesize::ByteSize;
use clap::{Arg, ArgMatches};
use errors::*;
use output::{Output, p2s};
use registry::{GCRoots, NullGCRoots, Register};
use statistics::Statistics;
use std::path::PathBuf;
use std::result;

static GC_PREFIX: &str = "/nix/var/nix/gcroots/profiles/per-user";

#[derive(Debug, Clone)]
pub struct App {
    startdir: PathBuf,
    quickcheck: ByteSize,
    register: bool,
    output: Output,
    cachefile: Option<PathBuf>,
    detailed_statistics: bool,
}

impl App {
    fn walker(&self) -> Result<ignore::WalkBuilder> {
        let startdir = self.startdir.canonicalize().chain_err(|| {
            format!("start dir {} is not accessible", p2s(&self.startdir))
        })?;
        let mut wb = ignore::WalkBuilder::new(&startdir);
        wb.parents(false)
            .git_exclude(false)
            .git_global(false)
            .git_ignore(false)
            .hidden(false);
        Ok(wb)
    }

    fn scanner(&self) -> scan::Scanner {
        scan::Scanner::new(self.quickcheck.as_usize())
    }

    fn gcroots(&self) -> Result<Box<Register>> {
        if self.register {
            Ok(Box::new(
                GCRoots::new(GC_PREFIX, &self.startdir, &self.output)?,
            ))
        } else {
            Ok(Box::new(NullGCRoots::new(&self.output)))
        }
    }

    fn cache(&self) -> Result<Cache> {
        match self.cachefile {
            Some(ref f) => Cache::new().open(f),
            None => Ok(Cache::new()),
        }
    }

    fn statistics(&self) -> Statistics {
        Statistics::new(self.detailed_statistics)
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            startdir: PathBuf::new(),
            quickcheck: ByteSize::kib(64),
            register: false,
            output: Output::default(),
            cachefile: None,
            detailed_statistics: false,
        }
    }
}

impl<'a> From<ArgMatches<'a>> for App {
    fn from(a: ArgMatches) -> Self {
        let output = Output::new(
            a.is_present("v"),
            a.is_present("d"),
            a.is_present("1"),
            a.value_of("C"),
            a.is_present("l") || a.is_present("R"),
        ).log_init();

        App {
            startdir: a.value_of_os("DIRECTORY").unwrap_or_default().into(),
            quickcheck: ByteSize::kib(a.value_of_lossy("q").unwrap().parse::<usize>().unwrap()),
            register: !a.is_present("R"),
            cachefile: a.value_of_os("CACHEFILE").map(PathBuf::from),
            detailed_statistics: a.is_present("s"),
            output,
        }
    }
}

fn parse_args() -> App {
    let a = |short, long, help| Arg::with_name(short).short(short).long(long).help(help);
    let kb_val = |s: String| -> result::Result<(), String> {
        s.parse::<u32>().map(|_| ()).map_err(|e| e.to_string())
    };
    let cachefile_val = |s: String| -> result::Result<(), String> {
        if s.ends_with(".json") || s.ends_with("json.gz") {
            Ok(())
        } else {
            Err(format!(
                "extension must be either {} or {}",
                p2s(".json"),
                p2s(".json.gz")
            ))
        }
    };

    clap::App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(Arg::with_name("DIRECTORY").required(true).help(
            "Start scan in DIRECTORY",
        ))
        .arg(
            a("l", "list", "Shows files containing Nix store references").display_order(1),
        )
        .arg(
            Arg::with_name("CACHEFILE")
                .short("c")
                .long("cache")
                .help(
                    "Caches scan results in CACHEFILE to avoid re-scanning unchanged files. \
                     File extension must be one of `.json` or `.json.gz`.",
                )
                .takes_value(true)
                .validator(cachefile_val),
        )
        .arg(
            a(
                "R",
                "no-register",
                "Don't register found references, implies --list",
            ).display_order(2),
        )
        .arg(a(
            "1",
            "oneline",
            "Prints each file with references on a single line",
        ))
        .arg(
            a("C", "color", "Funky colorful output")
                .takes_value(true)
                .possible_values(&["always", "never", "auto"])
                .default_value("auto")
                .long_help(
                    "Turns on funky colorful output. If set to \"auto\", color is on if \
                     run in a terminal.",
                ),
        )
        .arg(a(
            "s",
            "statistics",
            "Prints detailed statistics like scans per file type.",
        ))
        .arg(a("v", "verbose", "Additional output"))
        .arg(a(
            "d",
            "debug",
            "Prints every file opened, implies --verbose",
        ))
        .arg(
            a(
                "q",
                "quickcheck",
                "Give up if no Nix store reference is found in the first <q> kbytes of a file",
            ).takes_value(true)
                .default_value("64")
                .validator(kb_val),
        )
        .get_matches()
        .into()
}

fn main() {
    match walk::run(parse_args()) {
        Err(ref err) => {
            error!("{}", output::fmt_error_chain(err));
            std::process::exit(2)
        }
        Ok(exitcode) => std::process::exit(exitcode),
    }
}
