/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::fs;
use std::path::{Path, PathBuf};

pub fn get_files_with_extension(path: &Path, extension: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            result.append(&mut get_files_with_extension(&path, extension));
        } else if let Some(ext) = path.extension() {
            if ext == extension {
                result.push(path);
            }
        }
    }
    result.sort();
    result
}
