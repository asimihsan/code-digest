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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileKind {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub kind: FileKind,
    pub depth: isize,
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for File {}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

pub struct FileIterator {
    walker: ignore::Walk,
    path: PathBuf,
}

impl Iterator for FileIterator {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.walker.next() {
                Some(Ok(entry)) => {
                    let subpath = entry.path();
                    let relative_path = subpath.strip_prefix(&self.path).unwrap();
                    let depth = relative_path.components().count() as isize;
                    let file = File {
                        path: subpath.to_path_buf(),
                        kind: if subpath.is_dir() {
                            FileKind::Directory
                        } else {
                            FileKind::File
                        },
                        depth,
                    };
                    return Some(file);
                }
                Some(Err(err)) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
                None => {
                    return None;
                }
            }
        }
    }
}

pub fn get_files(path: PathBuf, ignore_dirs: &[PathBuf]) -> FileIterator {
    let mut builder = WalkBuilder::new(path.clone());
    builder
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .sort_by_file_path(|a, b| a.cmp(b));

    let mut override_builder = OverrideBuilder::new(path.clone());
    for ignore_dir in ignore_dirs {
        override_builder
            .add(&format!("!{}", ignore_dir.to_str().unwrap()))
            .unwrap();
    }
    override_builder.add("!.gitkeep").unwrap();
    builder.overrides(override_builder.build().unwrap());

    let walker = builder.build();
    FileIterator { walker, path }
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
    pub fn new_from_strings(glob_patterns: &[String]) -> Result<Self, glob::PatternError> {
        let mut result = GlobPatternMatcher::new();
        for glob_pattern in glob_patterns {
            result.add_glob_pattern(glob_pattern)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_glob_pattern_matcher() {
        let mut glob_pattern_matcher = GlobPatternMatcher::new();
        glob_pattern_matcher
            .add_glob_pattern("*.rs")
            .expect("Failed to add glob pattern");
        glob_pattern_matcher
            .add_glob_pattern("*.toml")
            .expect("Failed to add glob pattern");
        glob_pattern_matcher
            .add_glob_pattern("*.txt")
            .expect("Failed to add glob pattern");
        assert!(glob_pattern_matcher.matches(Path::new("Cargo.toml")));
        assert!(glob_pattern_matcher.matches(Path::new("src/main.rs")));
        assert!(glob_pattern_matcher.matches(Path::new("src/file_tree.rs")));
        assert!(glob_pattern_matcher.matches(Path::new("src/file_tree.txt")));
        assert!(!glob_pattern_matcher.matches(Path::new("src/file_tree.rs.bak")));
        assert!(glob_pattern_matcher.matches(Path::new("src/file_tree.rs.bak.txt")));
    }

    #[test]
    fn test_glob_pattern_matches_absolute_path() {
        let mut glob_pattern_matcher = GlobPatternMatcher::new();
        glob_pattern_matcher
            .add_glob_pattern("*.rs")
            .expect("Failed to add glob pattern");
        assert!(glob_pattern_matcher.matches(Path::new("/home/user/src/main.rs")));
    }

    /// Given
    /// a
    /// a/file_a1.txt
    /// a/file_a2.txt
    /// b
    /// b/file_b1.txt
    ///
    /// Return files in this order, including directories, with correct depths (1 for
    /// a, 2 for a/a_1.txt, etc.). Root is not included in the result.
    #[test]
    fn test_get_files() {
        // Create a temporary directory with a specific structure
        let temp_dir = tempdir().unwrap();
        let dir_a = temp_dir.path().join("a");
        let dir_b = temp_dir.path().join("b");
        let file_a1 = dir_a.join("file_a1.txt");
        let file_a2 = dir_a.join("file_a2.txt");
        let file_b1 = dir_b.join("file_b1.txt");

        std::fs::create_dir(&dir_a).unwrap();
        std::fs::create_dir(&dir_b).unwrap();
        std::fs::File::create(file_a1.clone()).unwrap();
        std::fs::File::create(file_a2.clone()).unwrap();
        std::fs::File::create(file_b1.clone()).unwrap();

        let ignore_dirs = Vec::new();

        let files = get_files(temp_dir.path().to_path_buf(), &ignore_dirs);
        let files: Vec<_> = files.collect();

        assert_eq!(files.len(), 6);
        assert_eq!(files[0].path, temp_dir.path().to_path_buf());
        assert_eq!(files[0].kind, FileKind::Directory);
        assert_eq!(files[0].depth, 0);
        assert_eq!(files[1].path, dir_a);
        assert_eq!(files[1].kind, FileKind::Directory);
        assert_eq!(files[1].depth, 1);
        assert_eq!(files[2].path, file_a1);
        assert_eq!(files[2].kind, FileKind::File);
        assert_eq!(files[2].depth, 2);
        assert_eq!(files[3].path, file_a2);
        assert_eq!(files[3].kind, FileKind::File);
        assert_eq!(files[3].depth, 2);
        assert_eq!(files[4].path, dir_b);
        assert_eq!(files[4].kind, FileKind::Directory);
        assert_eq!(files[4].depth, 1);
        assert_eq!(files[5].path, file_b1);
        assert_eq!(files[5].kind, FileKind::File);
        assert_eq!(files[5].depth, 2);
    }
}
