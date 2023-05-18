/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

// reference: https://github.com/Wilfred/difftastic/blob/84af470128adf82302d47749ab9dc33e0e6409b2/build.rs

use rayon::prelude::*;
use std::path::{Path, PathBuf};

struct TreeSitterParser {
    name: &'static str,
    src_dir: &'static Path,
    extra_files: Vec<&'static str>,
}

impl TreeSitterParser {
    fn build(&self) {
        let dir = PathBuf::from(&self.src_dir);

        let mut c_files = vec!["parser.c"];
        let mut cpp_files = vec![];

        for file in &self.extra_files {
            if file.ends_with(".c") {
                c_files.push(file);
            } else {
                cpp_files.push(file);
            }
        }

        if !cpp_files.is_empty() {
            let mut cpp_build = cc::Build::new();
            cpp_build
                .include(&dir)
                .cpp(true)
                .flag_if_supported("-Wno-implicit-fallthrough")
                .flag_if_supported("-Wno-unused-parameter")
                .flag_if_supported("-Wno-ignored-qualifiers")
                // Ignore warning from tree-sitter-html.
                .flag_if_supported("-Wno-sign-compare")
                // Ignore warning from tree-sitter-ruby.
                .flag_if_supported("-Wno-parentheses")
                // Ignore warning from tree-sitter-ruby.
                .flag_if_supported("-Wno-unused-but-set-variable")
                // Workaround for: https://github.com/ganezdragon/tree-sitter-perl/issues/16
                // should be removed after fixed.
                .flag_if_supported("-Wno-return-type");

            if cfg!(windows) {
                cpp_build.flag("/std:c++14");
            } else {
                cpp_build.flag("--std=c++14");
            }

            for file in cpp_files {
                cpp_build.file(dir.join(file));
            }

            cpp_build.compile(&format!("{}-cpp", self.name));
        }

        let mut build = cc::Build::new();
        if cfg!(target_env = "msvc") {
            build.flag("/utf-8");
        }
        build.include(&dir).warnings(false); // ignore unused parameter warnings
        for file in c_files {
            build.file(dir.join(file));
        }

        build.compile(self.name);
    }
}

fn main() {
    let parsers = vec![
        TreeSitterParser {
            name: "tree-sitter-go",
            src_dir: Path::new("../vendor/tree-sitter-go/src"),
            extra_files: vec![],
        },
        TreeSitterParser {
            name: "tree-sitter-hcl",
            src_dir: Path::new("../vendor/tree-sitter-hcl/src"),
            extra_files: vec!["scanner.cc"],
        },
        TreeSitterParser {
            name: "tree-sitter-java",
            src_dir: Path::new("../vendor/tree-sitter-java/src"),
            extra_files: vec![],
        },
        TreeSitterParser {
            name: "tree-sitter-python",
            src_dir: Path::new("../vendor/tree-sitter-python/src"),
            extra_files: vec!["scanner.cc"],
        },
        TreeSitterParser {
            name: "tree-sitter-rust",
            src_dir: Path::new("../vendor/tree-sitter-rust/src"),
            extra_files: vec!["scanner.c"],
        },
    ];

    // Only rerun if relevant files in the vendored_parsers/ directory change.
    for parser in &parsers {
        println!(
            "cargo:rerun-if-changed={}",
            parser.src_dir.canonicalize().unwrap().to_str().unwrap()
        );
    }

    parsers.par_iter().for_each(|p| p.build());
}
