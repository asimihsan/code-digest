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

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;

pub fn get_files_with_extension(path: PathBuf, extension: &str, ignore_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut builder = WalkBuilder::new(path.clone());
    builder.git_ignore(true)
        .git_global(false)
        .git_exclude(false);
    
    let mut override_builder = OverrideBuilder::new(path);
    for ignore_dir in ignore_dirs {
        override_builder.add(&format!("!{}", ignore_dir.to_str().unwrap())).unwrap();
    }
    builder.overrides(override_builder.build().unwrap());

    let walker = builder.build();

    for entry in walker {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        result.push(path.to_path_buf());
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
    }

    result.sort();
    result
}
