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
use language_parsers::{default_parse_config_for_language, parse, Language, ParseConfig};

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

pub struct ParseConfigs {
    go: ParseConfig,
    javascript: ParseConfig,
    json: ParseConfig,
    rust: ParseConfig,
    tsx: ParseConfig,
    typescript: ParseConfig,
}

impl ParseConfigs {
    pub fn new() -> Self {
        Self {
            go: default_parse_config_for_language(Language::Go),
            javascript: default_parse_config_for_language(Language::JavaScript),
            json: default_parse_config_for_language(Language::Json),
            rust: default_parse_config_for_language(Language::Rust),
            tsx: default_parse_config_for_language(Language::Tsx),
            typescript: default_parse_config_for_language(Language::TypeScript),
        }
    }

    fn for_extension(
        &self,
        extension: &str,
    ) -> Result<(&ParseConfig, &'static str), FileProcessorError> {
        match extension {
            "go" => Ok((&self.go, "go")),
            "js" | "mjs" | "cjs" => Ok((&self.javascript, "javascript")),
            "json" => Ok((&self.json, "json")),
            "rs" => Ok((&self.rust, "rust")),
            "ts" => Ok((&self.typescript, "typescript")),
            "tsx" => Ok((&self.tsx, "tsx")),
            _ => Err(FileProcessorError::UnsupportedFileKind(
                extension.to_string(),
            )),
        }
    }
}

pub fn process_files<'a>(
    files: impl Iterator<Item = file_system::File> + 'a,
    parse_configs: &'a ParseConfigs,
    glob_matcher: &'a GlobPatternMatcher,
) -> impl Iterator<Item = Result<String, FileProcessorError>> + 'a {
    files.into_iter().filter_map(move |file| {
        if file.kind != file_system::FileKind::File {
            return None;
        }
        Some(process_file(&file.path, parse_configs, glob_matcher))
    })
}

pub fn process_file(
    file_path: &Path,
    parse_configs: &ParseConfigs,
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
    let (parse_config, code_fence_language) = parse_configs.for_extension(extension)?;
    let parsed = parse(&source_code, parse_config);
    if parsed.is_err() {
        return Err(FileProcessorError::ErrorParsingFile(parsed.err().unwrap()));
    }
    let parsed = parsed.unwrap();

    output.push_str(&format!("`{}`\n", file_path.display()));
    output.push_str(&format!("```{}\n", code_fence_language));

    for (i, key_content) in parsed.iter().enumerate() {
        output.push_str(&key_content.content.to_string());
        output.push('\n');
        if i < parsed.len() - 1 {
            output.push('\n');
        }
    }
    output.push_str("```\n");

    Ok(output)
}

#[cfg(test)]
mod tests {
    use file_system::{File, FileKind};

    use crate::GlobPatternMatcher;

    use super::*;

    #[test]
    fn test_process_file_rust() {
        let parse_configs = ParseConfigs::new();
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

        let result = process_file(&file_path, &parse_configs, &glob_matcher);
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
        let parse_configs = ParseConfigs::new();
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
            process_files(files.into_iter(), &parse_configs, &glob_matcher).collect();

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

    #[test]
    fn test_process_file_javascript_typescript_and_json() {
        let parse_configs = ParseConfigs::new();
        let glob_matcher = GlobPatternMatcher::new_from_strings(&[]).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();

        let javascript_path = temp_dir.path().join("example.js");
        std::fs::write(
            &javascript_path,
            "import { useMemo } from 'react';\n\nexport function render(value) {\n  return value + 1;\n}\n",
        )
        .unwrap();

        let typescript_path = temp_dir.path().join("example.ts");
        std::fs::write(
            &typescript_path,
            "export interface User {\n  id: string;\n}\n\nexport function formatUser(user: User): string {\n  return user.id;\n}\n",
        )
        .unwrap();

        let tsx_path = temp_dir.path().join("example.tsx");
        std::fs::write(
            &tsx_path,
            "export function View(): JSX.Element {\n  return <section>Hello</section>;\n}\n",
        )
        .unwrap();

        let json_path = temp_dir.path().join("example.json");
        std::fs::write(
            &json_path,
            "{\n  \"name\": \"code-digest\",\n  \"enabled\": true\n}\n",
        )
        .unwrap();

        let javascript_output =
            process_file(&javascript_path, &parse_configs, &glob_matcher).unwrap();
        let typescript_output =
            process_file(&typescript_path, &parse_configs, &glob_matcher).unwrap();
        let tsx_output = process_file(&tsx_path, &parse_configs, &glob_matcher).unwrap();
        let json_output = process_file(&json_path, &parse_configs, &glob_matcher).unwrap();

        assert_eq!(
            javascript_output,
            format!(
                "`{}`\n```javascript\nimport {{ useMemo }} from 'react';\n\nfunction render(value) {{\n  // ...\n}}\n```\n",
                javascript_path.display()
            )
        );
        assert_eq!(
            typescript_output,
            format!(
                "`{}`\n```typescript\ninterface User {{\n  id: string;\n}}\n\nfunction formatUser(user: User): string {{\n  // ...\n}}\n```\n",
                typescript_path.display()
            )
        );
        assert_eq!(
            tsx_output,
            format!(
                "`{}`\n```tsx\nfunction View(): JSX.Element {{\n  // ...\n}}\n```\n",
                tsx_path.display()
            )
        );
        assert_eq!(
            json_output,
            format!(
                "`{}`\n```json\n{{\n  \"name\": \"code-digest\",\n  \"enabled\": true\n}}\n```\n",
                json_path.display()
            )
        );
    }
}
