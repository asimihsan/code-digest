/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use file_system::File;

#[derive(thiserror::Error, Debug)]
pub enum FileTreeError {
    #[error("Error getting files: {0}")]
    ErrorGettingFiles(#[from] std::io::Error),
}

/// Arguments passed to the callback function.
#[derive(Debug, Clone, Copy)]
pub struct CallbackArgs<T>
where
    T: AsRef<str>,
{
    /// The output to print.
    pub output: T,

    /// Whether to add a linebreak after the output.
    pub linebreak: bool,
}

/// Print an indented file and directory tree.
///
/// Note that the File struct is
/// pub struct File {
//     pub path: PathBuf,
//     pub kind: FileKind,
//     pub depth: isize,
// }
//
// pub enum FileKind {
//     File,
//     Directory,
// }
//
// This method does not need to do any file system traversal or depth determination.
pub fn print_file_tree(
    files: impl Iterator<Item = File>,
    mut callback: impl FnMut(CallbackArgs<&str>),
) -> Result<(), FileTreeError> {
    for (i, file) in files.into_iter().enumerate() {
        if i == 0 {
            callback(CallbackArgs {
                output: ".",
                linebreak: true,
            });
            continue;
        }

        // TODO implement this function
        print_indent(&mut callback, file.depth, false, false);

        callback(CallbackArgs {
            output: file.path.file_name().unwrap().to_str().unwrap(),
            linebreak: true,
        });
    }
    Ok(())
}

fn print_indent(
    callback: &mut impl FnMut(CallbackArgs<&'static str>),
    depth: isize,
    is_last_sibling: bool,
    is_last_entry: bool,
) {
    for _ in 0..depth - 1 {
        if is_last_sibling {
            callback(CallbackArgs {
                output: "    ",
                linebreak: false,
            })
        } else {
            callback(CallbackArgs {
                output: "│   ",
                linebreak: false,
            })
        }
    }
    if depth > 0 {
        if is_last_entry {
            callback(CallbackArgs {
                output: "└── ",
                linebreak: false,
            })
        } else {
            callback(CallbackArgs {
                output: "├── ",
                linebreak: false,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;
    use std::fs::File;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_print_file_tree() {
        // Create a temporary directory with a specific structure
        let temp_dir = tempdir().unwrap();
        let dir_a = temp_dir.path().join("a");
        let dir_b = temp_dir.path().join("b");
        let file_a1 = dir_a.join("file_a1.txt");
        let file_a2 = dir_a.join("file_a2.txt");
        let file_b1 = dir_b.join("file_b1.txt");

        std::fs::create_dir(&dir_a).unwrap();
        std::fs::create_dir(&dir_b).unwrap();
        File::create(file_a1).unwrap();
        File::create(file_a2).unwrap();
        File::create(file_b1).unwrap();

        let files = file_system::get_files(temp_dir.into_path(), &[]);

        let mut output = String::new();

        let result = print_file_tree(
            files,
            |CallbackArgs {
                 output: s,
                 linebreak,
             }| {
                output.push_str(s);
                if linebreak {
                    output.push('\n');
                }
            },
        );
        assert!(result.is_ok());

        let expected_output_1 = "\
.
├── a
│   ├── file_a1.txt
│   ├── file_a2.txt
├── b
│   ├── file_b1.txt
";

        // could also be b first
        let expected_output_2 = "\
.
├── b
│   ├── file_b1.txt
├── a
│   ├── file_a1.txt
│   ├── file_a2.txt
";

        println!("output: {}", output);

        let expected_output =
            expected_output_1.to_string() == output || expected_output_2.to_string() == output;
        assert!(expected_output);
    }
}
