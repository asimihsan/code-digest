/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

mod file_processor;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

use std::path::{Path, PathBuf};

use crate::file_processor::process_file;
use clap::Parser;
use file_system::{get_files, GlobPatternMatcher};
use language_parsers::{default_parse_config_for_language, parse, ParseConfig};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the directory containing the files.
    directory: PathBuf,

    /// Additional directories to ignore (optional, zero or more)
    #[clap(short = 'i', long)]
    ignore: Vec<PathBuf>,

    /// Glob patterns for which to include the full file contents, e.g. `*.md` (optional, zero or more)
    #[clap(short = 'I', long)]
    include: Vec<String>,
}

pub fn main() {
    let cli = Cli::parse();
    if !cli.directory.is_dir() {
        eprintln!("Not a directory: {}", cli.directory.display());
        std::process::exit(1);
    }
    let directory = cli.directory;

    let ignore_dirs: Vec<PathBuf> = cli
        .ignore
        .iter()
        .flat_map(|values| {
            values
                .iter()
                .map(|dir| shellexpand::full(dir.to_str().unwrap()).unwrap())
                .map(|dir| PathBuf::from(dir.to_string()))
                .collect::<Vec<PathBuf>>()
        })
        .collect();
    let glob_matcher = GlobPatternMatcher::new_from_strings(cli.include).unwrap();

    let files = get_files(directory, &ignore_dirs);
    let go_config = default_parse_config_for_language(language_parsers::Language::Go);
    let rust_config = default_parse_config_for_language(language_parsers::Language::Rust);

    for (i, file_path) in files.iter().enumerate() {
        process_file(&file_path, &go_config, &rust_config, &glob_matcher, |s| {
            println!("{}", s);
        });
        if i < files.len() - 1 {
            println!();
        }
    }
}
