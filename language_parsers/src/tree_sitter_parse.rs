// reference: https://github.com/Wilfred/difftastic/blob/84af470128adf82302d47749ab9dc33e0e6409b2/src/parse/tree_sitter_parser.rs

use std::collections::VecDeque;
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

fn test_rust_parse() {
    let source_code = r#"
// reference: https://github.com/Wilfred/difftastic/blob/84af470128adf82302d47749ab9dc33e0e6409b2/src/parse/tree_sitter_parser.rs

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

fn test_rust_parse() {
    let source_code = "fn test() {}";
    let config = from_language(Language::Rust);
    let tree = to_tree(source_code, &config).unwrap();
    let root_node = tree.root_node();

    assert_eq!(root_node.kind(), "source_file");
    assert_eq!(root_node.start_position().column, 0);
    assert_eq!(root_node.end_position().column, 12);

    let cursor = &mut root_node.walk();
    root_node.children(cursor).for_each(|child| {
        println!("{}: {}", child.kind(), child.start_position().column);
    });
}

#[cfg(test)]
mod tests {
    use crate::tree_sitter_parse::test_rust_parse;

    #[test]
    fn test_parse() {
        test_rust_parse();
    }
}
"#;
    let config = from_language(Language::Rust);
    let tree = to_tree(source_code, &config).unwrap();
    let root_node = tree.root_node();

    let cursor = &mut root_node.walk();
    let mut queue: VecDeque<ts::Node> = VecDeque::new();
    queue.push_back(root_node);
    loop {
        if queue.is_empty() {
            break;
        }
        let node = queue.pop_front().unwrap();
        println!(
            "{}: {}:{} to {}:{}",
            node.kind(),
            node.start_position().row,
            node.start_position().column,
            node.end_position().row,
            node.end_position().column,
        );
        node.children(cursor).for_each(|child| {
            queue.push_back(child);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::tree_sitter_parse::test_rust_parse;

    #[test]
    fn test_parse() {
        test_rust_parse();
    }
}
