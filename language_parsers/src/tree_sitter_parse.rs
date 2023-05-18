// reference: https://github.com/Wilfred/difftastic/blob/84af470128adf82302d47749ab9dc33e0e6409b2/src/parse/tree_sitter_parser.rs

use std::collections::{HashSet, VecDeque};

use tree_sitter as ts;

extern "C" {
    fn tree_sitter_go() -> ts::Language;
    fn tree_sitter_hcl() -> ts::Language;
    fn tree_sitter_java() -> ts::Language;
    fn tree_sitter_python() -> ts::Language;
    fn tree_sitter_rust() -> ts::Language;
}

pub struct TreeSitterConfig {
    pub language: ts::Language,
}

pub enum Language {
    Go,
    Hcl,
    Java,
    Python,
    Rust,
}

// from enum Language to TreeSitterConfig
pub fn from_language(language: Language) -> TreeSitterConfig {
    match language {
        Language::Go => TreeSitterConfig {
            language: unsafe { tree_sitter_go() },
        },
        Language::Hcl => TreeSitterConfig {
            language: unsafe { tree_sitter_hcl() },
        },
        Language::Java => TreeSitterConfig {
            language: unsafe { tree_sitter_java() },
        },
        Language::Python => TreeSitterConfig {
            language: unsafe { tree_sitter_python() },
        },
        Language::Rust => TreeSitterConfig {
            language: unsafe { tree_sitter_rust() },
        },
    }
}

pub fn to_tree(src: &str, config: &TreeSitterConfig) -> Option<ts::Tree> {
    let mut parser = ts::Parser::new();
    parser
        .set_language(config.language)
        .expect("Incompatible tree-sitter version");
    parser.parse(src, None)
}
