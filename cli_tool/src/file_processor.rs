/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use file_system::GlobPatternMatcher;
use language_parsers::{parse, ParseConfig};
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum FileProcessorError {
    #[error("Error reading file: {0}")]
    ErrorReadingFile(#[from] std::io::Error),
}

pub fn process_files(
    files: &[file_system::File],
    go_config: &ParseConfig,
    rust_config: &ParseConfig,
    glob_matcher: &GlobPatternMatcher,
) {
    for (i, file) in files.iter().enumerate() {
        if file.kind != file_system::FileKind::File {
            continue;
        }
        if let Err(e) = process_file(&file.path, go_config, rust_config, glob_matcher, |s| {
            println!("{}", s);
        }) {
            eprintln!("Error processing file {}: {}", file.path.display(), e);
        }
        if i < files.len() - 1 {
            println!();
        }
    }
}

pub fn process_file(
    file_path: &Path,
    go_config: &ParseConfig,
    rust_config: &ParseConfig,
    glob_matcher: &GlobPatternMatcher,
    mut callback: impl FnMut(&str),
) -> Result<(), FileProcessorError> {
    let source_code =
        std::fs::read_to_string(file_path).map_err(FileProcessorError::ErrorReadingFile)?;

    if glob_matcher.matches(file_path) {
        callback(&format!("`{}`", file_path.display()));
        callback(&format!("```\n{}\n```\n", source_code));
        return Ok(());
    }

    let extension = file_path.extension();
    if extension.is_none() {
        return Ok(());
    }
    let extension = extension.unwrap().to_str().unwrap();
    let result = match extension {
        "go" => parse(&source_code, go_config).unwrap_or_else(|e| {
            eprintln!("Error parsing file: {}", e);
            std::process::exit(1);
        }),
        "rs" => parse(&source_code, rust_config).unwrap_or_else(|e| {
            eprintln!("Error parsing file: {}", e);
            std::process::exit(1);
        }),
        _ => return Ok(()),
    };

    callback(&format!("`{}`", file_path.display()));
    match extension {
        "go" => {
            callback("```go");
        }
        "rs" => {
            callback("```rust");
        }
        _ => {
            eprintln!("Unknown extension: {}", extension);
            std::process::exit(1);
        }
    }

    for (i, key_content) in result.iter().enumerate() {
        callback(&key_content.content.to_string());
        if i < result.len() - 1 {
            callback("\n");
        }
    }
    callback("```");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GlobPatternMatcher;
    use language_parsers::{default_parse_config_for_language, Language};

    #[test]
    fn test_process_file_rust() {
        let rust_config = default_parse_config_for_language(Language::Rust);
        let go_config = default_parse_config_for_language(Language::Go);
        let glob_matcher = GlobPatternMatcher::new_from_strings(&vec![]).unwrap();

        // Create a temporary file with Rust code
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(
            &file_path,
            r#"
fn main() {
    println!("Hello, world!");
}
"#,
        )
        .unwrap();

        let mut actual_output = String::new();
        process_file(&file_path, &go_config, &rust_config, &glob_matcher, |s| {
            actual_output.push_str(s);
            actual_output.push('\n');
        })
        .expect("Error processing file");

        let expected_output = format!(
            r#"`{}`
```rust
fn main() {{
    // ...
}}
```
"#,
            file_path.display()
        );

        assert_eq!(actual_output, expected_output);
    }
}
