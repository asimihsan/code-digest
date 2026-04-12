/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

// reference: https://github.com/Wilfred/difftastic/blob/84af470128adf82302d47749ab9dc33e0e6409b2/src/parse/tree_sitter_parser.rs

use tree_sitter as ts;
use tree_sitter_language::LanguageFn;

use crate::Language;

extern "C" {
    fn tree_sitter_go() -> *const ();
    fn tree_sitter_hcl() -> *const ();
    fn tree_sitter_java() -> *const ();
    fn tree_sitter_javascript() -> *const ();
    fn tree_sitter_json() -> *const ();
    fn tree_sitter_python() -> *const ();
    fn tree_sitter_rust() -> *const ();
    fn tree_sitter_tsx() -> *const ();
    fn tree_sitter_typescript() -> *const ();
}

pub struct TreeSitterConfig {
    pub language: ts::Language,
}

fn language_from_raw(raw: unsafe extern "C" fn() -> *const ()) -> ts::Language {
    ts::Language::new(unsafe { LanguageFn::from_raw(raw) })
}

// from enum Language to TreeSitterConfig
pub fn from_language(language: Language) -> TreeSitterConfig {
    match language {
        Language::Go => TreeSitterConfig {
            language: language_from_raw(tree_sitter_go),
        },
        Language::Hcl => TreeSitterConfig {
            language: language_from_raw(tree_sitter_hcl),
        },
        Language::Java => TreeSitterConfig {
            language: language_from_raw(tree_sitter_java),
        },
        Language::JavaScript => TreeSitterConfig {
            language: language_from_raw(tree_sitter_javascript),
        },
        Language::Json => TreeSitterConfig {
            language: language_from_raw(tree_sitter_json),
        },
        Language::Python => TreeSitterConfig {
            language: language_from_raw(tree_sitter_python),
        },
        Language::Rust => TreeSitterConfig {
            language: language_from_raw(tree_sitter_rust),
        },
        Language::Tsx => TreeSitterConfig {
            language: language_from_raw(tree_sitter_tsx),
        },
        Language::TypeScript => TreeSitterConfig {
            language: language_from_raw(tree_sitter_typescript),
        },
    }
}

pub fn to_tree(src: &str, config: &TreeSitterConfig) -> Option<ts::Tree> {
    let mut parser = ts::Parser::new();
    parser
        .set_language(&config.language)
        .expect("Incompatible tree-sitter version");
    parser.parse(src, None)
}
