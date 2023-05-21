/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::path::PathBuf;

use file_system::{get_files, GlobPatternMatcher};
use language_parsers::default_parse_config_for_language;

use crate::file_processor::{process_files, FileProcessorError};
use crate::file_tree::{print_file_tree, CallbackArgs};

mod config;
mod file_processor;
mod file_tree;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = config::AppConfig::new(&args).unwrap_or_else(|e| {
        eprintln!("Error parsing CLI arguments: {}", e);
        std::process::exit(1);
    });

    let directory = shellexpand::full(config.directory.as_str())
        .map_err(|e| {
            eprintln!("Error expanding directory: {}", e);
            std::process::exit(1);
        })
        .unwrap();
    let directory = PathBuf::from(directory.as_ref());
    if !directory.is_dir() {
        eprintln!("Not a directory: {}", &config.directory);
        std::process::exit(1);
    }

    let ignore_dirs: &Vec<PathBuf> = &config
        .ignore
        .iter()
        .map(|dir| shellexpand::full(dir.to_str().unwrap()).unwrap())
        .map(|dir| PathBuf::from(dir.to_string()))
        .collect::<Vec<PathBuf>>();

    // cli.include comes from a shell and should not include single quotes around e.g. '*.md'. But
    // if it does then we remove them here. Must be a matching pair of single quotes at the start
    // and end of the string.
    let cli_include = &config
        .include
        .iter()
        .map(|s| {
            if s.starts_with('\'') && s.ends_with('\'') {
                s[1..s.len() - 1].to_string()
            } else {
                s.to_string()
            }
        })
        .collect::<Vec<String>>();

    let glob_matcher = GlobPatternMatcher::new_from_strings(cli_include).unwrap();

    let go_config = default_parse_config_for_language(language_parsers::Language::Go);
    let rust_config = default_parse_config_for_language(language_parsers::Language::Rust);

    if config.tree {
        print_file_tree(
            get_files(directory.clone(), ignore_dirs),
            |CallbackArgs {
                 output: s,
                 linebreak,
             }| {
                print!("{}", s);
                if linebreak {
                    println!();
                }
            },
        )
        .unwrap_or_else(|e| {
            eprintln!("Error printing file tree: {}", e);
            std::process::exit(1);
        });
    }

    for file_result in process_files(
        get_files(directory, ignore_dirs),
        &go_config,
        &rust_config,
        &glob_matcher,
    ) {
        match file_result {
            Ok(file) => {
                println!("{}", file);
            }
            Err(FileProcessorError::UnsupportedFileKind(_)) => {}
            Err(FileProcessorError::FileSkipped(_)) => {}
            _ => {
                eprintln!("Error processing file: {:?}\n", file_result);
            }
        }
    }
}
