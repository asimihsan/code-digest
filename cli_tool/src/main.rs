/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

use std::collections::HashSet;
use std::path::PathBuf;

use clap::Parser;
use file_system::{get_files, GlobPatternMatcher};
use language_parsers::{default_parse_config_for_language, parse};

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
    if cli.directory.is_dir() == false {
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

    let files = get_files(directory.clone(), &ignore_dirs);
    let go_config = default_parse_config_for_language(language_parsers::Language::Go);
    let rust_config = default_parse_config_for_language(language_parsers::Language::Rust);

    for (file_number, file_path) in files.iter().enumerate() {
        if glob_matcher.matches(file_path) {
            println!("`{}`", file_path.display());
            let source_code = std::fs::read_to_string(&file_path).expect("Unable to read file");
            println!("```\n{}\n```\n", source_code);
            continue;
        }

        let source_code = std::fs::read_to_string(&file_path).expect("Unable to read file");

        let extension = file_path.extension();
        if extension.is_none() {
            continue;
        }
        let extension = extension.unwrap().to_str().unwrap();
        let result = match extension {
            "go" => parse(&source_code, &go_config).unwrap_or_else(|e| {
                eprintln!("Error parsing file: {}", e);
                std::process::exit(1);
            }),
            "rs" => parse(&source_code, &rust_config).unwrap_or_else(|e| {
                eprintln!("Error parsing file: {}", e);
                std::process::exit(1);
            }),
            _ => continue,
        };

        println!("`{}`", file_path.display());
        match extension {
            "go" => {
                println!("```go\n");
            }
            "rs" => {
                println!("```rust\n");
            }
            _ => {
                eprintln!("Unknown extension: {}", extension);
                std::process::exit(1);
            }
        }

        for (i, key_content) in result.iter().enumerate() {
            println!("{}", key_content.content);
            if i < result.len() - 1 {
                println!();
            }
        }
        println!("```\n");
        if file_number < files.len() - 1 {
            println!();
        }
    }
}
