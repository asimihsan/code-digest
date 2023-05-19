/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

pub fn print_file_tree(directory: &Path, ignore_dirs: &[PathBuf], mut callback: impl FnMut(&str)) {
    let mut stack = VecDeque::new();
    stack.push_back((directory.to_path_buf(), 0));
    while let Some((directory, depth)) = stack.pop_back() {
        if ignore_dirs.contains(&directory) {
            continue;
        }
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error getting files: {}", e);
                std::process::exit(1);
            }
        };
        for entry in entries {
            let path = match entry {
                Ok(entry) => entry.path(),
                Err(e) => {
                    eprintln!("Error getting file: {}", e);
                    std::process::exit(1);
                }
            };
            let is_dir = path.is_dir();
            print_indent(&mut callback, depth);
            callback(&format!("{}", path.file_name().unwrap().to_string_lossy()));
            if is_dir {
                stack.push_back((path, depth + 1));
            }
        }
    }
}

fn print_indent(mut callback: impl FnMut(&str), depth: usize) {
    for _ in 0..depth {
        callback(" ");
    }
}
