/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::path::Path;

use file_system::GlobPatternMatcher;
use language_parsers::{parse, ParseConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileSkipReason {
    FileExtension,
}

#[derive(thiserror::Error, Debug)]
pub enum FileProcessorError {
    #[error("Error reading file: {0}")]
    ErrorReadingFile(#[from] std::io::Error),

    #[error("File skipped: {0:?}")]
    FileSkipped(FileSkipReason),

    #[error("Error parsing file: {0}")]
    ErrorParsingFile(#[from] language_parsers::ParseError),

    #[error("Unsupported file kind: {0:?}")]
    UnsupportedFileKind(String),
}

pub fn process_files<'a>(
    files: &'a [file_system::File],
    go_config: &'a ParseConfig,
    rust_config: &'a ParseConfig,
    glob_matcher: &'a GlobPatternMatcher,
) -> impl Iterator<Item = Result<String, FileProcessorError>> + 'a {
    files.iter().filter_map(move |file| {
        if file.kind != file_system::FileKind::File {
            return None;
        }
        Some(process_file(
            &file.path,
            go_config,
            rust_config,
            glob_matcher,
        ))
    })
}

pub fn process_file(
    file_path: &Path,
    go_config: &ParseConfig,
    rust_config: &ParseConfig,
    glob_matcher: &GlobPatternMatcher,
) -> Result<String, FileProcessorError> {
    let source_code =
        std::fs::read_to_string(file_path).map_err(FileProcessorError::ErrorReadingFile)?;
    let mut output = String::new();

    if glob_matcher.matches(file_path) {
        output.push_str(&format!("`{}`", file_path.display()));
        output.push_str(&format!("```\n{}\n```\n", source_code));
        return Ok(output);
    }

    let extension = file_path.extension();
    if extension.is_none() {
        return Err(FileProcessorError::FileSkipped(
            FileSkipReason::FileExtension,
        ));
    }
    let extension = extension.unwrap().to_str().unwrap();
    let parse_config = match extension {
        "go" => go_config,
        "rs" => rust_config,
        _ => {
            return Err(FileProcessorError::UnsupportedFileKind(
                extension.to_string(),
            ))
        }
    };
    let parsed = parse(&source_code, parse_config);
    if parsed.is_err() {
        return Err(FileProcessorError::ErrorParsingFile(parsed.err().unwrap()));
    }
    let parsed = parsed.unwrap();

    output.push_str(&format!("`{}`\n", file_path.display()));
    match extension {
        "go" => {
            output.push_str("```go\n");
        }
        "rs" => {
            output.push_str("```rust\n");
        }
        _ => unreachable!(),
    }

    for key_content in &parsed {
        output.push_str(&key_content.content.to_string());
        output.push('\n');
    }
    output.push_str("```\n");

    Ok(output)
}

#[cfg(test)]
mod tests {
    use file_system::{File, FileKind};
    use language_parsers::{default_parse_config_for_language, Language};

    use crate::GlobPatternMatcher;

    use super::*;

    #[test]
    fn test_process_file_rust() {
        let rust_config = default_parse_config_for_language(Language::Rust);
        let go_config = default_parse_config_for_language(Language::Go);
        let glob_matcher = GlobPatternMatcher::new_from_strings(&[]).unwrap();

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

        let result = process_file(&file_path, &go_config, &rust_config, &glob_matcher);
        assert!(result.is_ok());
        let actual_output = result.unwrap();

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

    #[test]
    fn test_process_files() {
        let rust_config = default_parse_config_for_language(Language::Rust);
        let go_config = default_parse_config_for_language(Language::Go);
        let glob_matcher = GlobPatternMatcher::new_from_strings(&[]).unwrap();

        // Create a temporary file with Rust code
        let temp_dir = tempfile::tempdir().unwrap();
        let rust_file_path = temp_dir.path().join("test.rs");
        std::fs::write(
            &rust_file_path,
            r#"
fn main() {
    println!("Hello, world!");
}
"#,
        )
        .expect("Failed to write to temporary file");

        let go_file_path = temp_dir.path().join("test.go");
        std::fs::write(
            &go_file_path,
            r#"
package main

import "fmt"

func main() {
    fmt.Println("Hello, world!")
}
"#,
        )
        .expect("Failed to write to temporary file");

        let files = vec![
            File {
                path: rust_file_path.clone(),
                kind: FileKind::File,
                depth: 0,
            },
            File {
                path: go_file_path.clone(),
                kind: FileKind::File,
                depth: 0,
            },
        ];

        let results: Vec<_> =
            process_files(&files, &go_config, &rust_config, &glob_matcher).collect();

        assert_eq!(results.len(), 2);

        let rust_expected_output = format!(
            r#"`{}`
```rust
fn main() {{
    // ...
}}
```
"#,
            rust_file_path.display()
        );
        let go_expected_output = format!(
            r#"`{}`
```go
import "fmt"
func main() {{
	// ...
}}
```
"#,
            go_file_path.display()
        );

        assert_eq!(results[0].as_ref().unwrap(), &rust_expected_output);
        assert_eq!(results[1].as_ref().unwrap(), &go_expected_output);
    }
}
