/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::path::{Path, PathBuf};

use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;

pub fn get_files(path: PathBuf, ignore_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut builder = WalkBuilder::new(path.clone());
    builder
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false);

    let mut override_builder = OverrideBuilder::new(path);
    for ignore_dir in ignore_dirs {
        override_builder
            .add(&format!("!{}", ignore_dir.to_str().unwrap()))
            .unwrap();
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
                result.push(path.to_path_buf());
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

pub struct GlobPatternMatcher {
    glob_patterns: Vec<glob::Pattern>,
}

impl Default for GlobPatternMatcher {
    fn default() -> Self {
        GlobPatternMatcher::new()
    }
}

impl GlobPatternMatcher {
    pub fn new() -> Self {
        GlobPatternMatcher {
            glob_patterns: Vec::new(),
        }
    }
    pub fn new_from_strings(glob_patterns: Vec<String>) -> Result<Self, glob::PatternError> {
        let mut result = GlobPatternMatcher::new();
        for glob_pattern in glob_patterns {
            result.add_glob_pattern(&glob_pattern)?;
        }
        Ok(result)
    }

    pub fn add_glob_pattern(&mut self, glob_pattern: &str) -> Result<(), glob::PatternError> {
        let glob_pattern = glob::Pattern::new(glob_pattern)?;
        self.glob_patterns.push(glob_pattern);
        Ok(())
    }

    /// Returns true if the file path matches any of the glob patterns. Otherwise, returns false.
    pub fn matches(&self, file_path: &Path) -> bool {
        for glob_pattern in &self.glob_patterns {
            if glob_pattern.matches_path(file_path) {
                return true;
            }
        }
        false
    }
}
