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

use std::path::PathBuf;

use clap::Parser;
use file_system::get_files_with_extension;
use language_parsers::{default_parse_config_for_language, parse};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the directory containing the files.
    directory: PathBuf,

    /// Additional directories to ignore (optional, zero or more)
    #[clap(short, long)]
    ignore: Vec<PathBuf>,
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
        .map(|values| {
            values
                .iter()
                .map(|dir| shellexpand::full(dir.to_str().unwrap()).unwrap())
                .map(|dir| PathBuf::from(dir.to_string()))
                .collect::<Vec<PathBuf>>()
        })
        .flatten()
        .collect();

    let go_files = get_files_with_extension(directory.clone(), "go", &ignore_dirs);
    let go_config = default_parse_config_for_language(language_parsers::Language::Go);
    let rust_files = get_files_with_extension(directory, "rs", &ignore_dirs);
    let rust_config = default_parse_config_for_language(language_parsers::Language::Rust);
    let all_files: Vec<PathBuf> = go_files.iter().chain(rust_files.iter()).map(|p| p.to_path_buf()).collect();

    for (file_number, file_path) in all_files.iter().enumerate() {
        let source_code = std::fs::read_to_string(&file_path).expect("Unable to read file");
        let extension = file_path.extension().unwrap().to_str().unwrap();
        let result = match extension {
            "go" => parse(&source_code, &go_config).unwrap_or_else(|e| {
                eprintln!("Error parsing file: {}", e);
                std::process::exit(1);
            }),
            "rs" => parse(&source_code, &rust_config).unwrap_or_else(|e| {
                eprintln!("Error parsing file: {}", e);
                std::process::exit(1);
            }),
            _ => {
                todo!()
            }
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
        if file_number < all_files.len() - 1 {
            println!();
        }
    }
}
