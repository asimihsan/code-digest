/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use file_system::get_files_with_extension;
use language_parsers::{default_parse_config_for_language, parse};

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }

    let directory = std::path::Path::new(&args[1]);
    let go_files = get_files_with_extension(directory, "go");
    let config = default_parse_config_for_language(language_parsers::Language::Go);

    for (file_number, file_path) in go_files.iter().enumerate() {
        let source_code = std::fs::read_to_string(&file_path).expect("Unable to read file");
        let result = parse(&source_code, &config).unwrap_or_else(|e| {
            eprintln!("Error parsing file: {}", e);
            std::process::exit(1);
        });
        println!("`{}`\n\n", file_path.display());
        println!("```go\n");
        for (i, key_content) in result.iter().enumerate() {
            println!("{}", key_content.content);
            if i < result.len() - 1 {
                println!();
            }
        }
        println!("```");
        if file_number < go_files.len() - 1 {
            println!();
        }
    }
}
