/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::collections::{HashMap, VecDeque};

use tree_sitter as ts;

use crate::tree_sitter_parse::{from_language, to_tree};

mod tree_sitter_parse;

#[derive(Clone, Copy)]
pub enum Language {
    Go,
    Hcl,
    Java,
    Python,
    Rust,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("custom selector action failed")]
    CustomActionFailed(String),

    #[error("tree-sitter parse error")]
    TreeSitterParseError(#[from] tree_sitter::LanguageError),
}

type ParseResult<T, E = ParseError> = Result<T, E>;
type SelectorFunction =
    dyn Fn(ts::Node, &mut ts::TreeCursor, &str, &mut ParseState) -> ParseResult<String>;

// SelectorType lets you choose which tree-sitter AST nodes to select (traverse), which to capture,
// and if captured whether or not to elide the block contents. You need to select AST nodes that
// are parents of other nodes that you are interested in.
pub enum SelectorAction {
    SelectOnly,
    CaptureWithoutBlock,
    CaptureAll,
    Custom(Box<SelectorFunction>),
}

pub struct Selector {
    pub node_kind: String,
    pub action: SelectorAction,
}

impl Selector {
    pub fn new(node_kind: impl Into<String>, action: SelectorAction) -> Selector {
        Selector {
            node_kind: node_kind.into(),
            action,
        }
    }
}

pub enum Indentation {
    Tabs,
    Spaces(usize),
}

impl Default for Indentation {
    fn default() -> Self {
        Indentation::Spaces(4)
    }
}

pub struct ParseConfig {
    language_config: tree_sitter_parse::TreeSitterConfig,
    selectors: HashMap<String, Selector>,
    indent_value: String,
}

impl ParseConfig {
    pub fn new(language: Language, indentation: Indentation) -> ParseConfig {
        let indent_value = match indentation {
            Indentation::Tabs => "\t".to_string(),
            Indentation::Spaces(indent_size) => {
                let mut indent_value = String::with_capacity(indent_size);
                for _ in 0..indent_size {
                    indent_value.push(' ');
                }
                indent_value
            }
        };

        ParseConfig {
            language_config: from_language(language),
            selectors: HashMap::new(),
            indent_value,
        }
    }

    pub fn add_selector(&mut self, selector: Selector) {
        self.selectors.insert(selector.node_kind.clone(), selector);
    }

    pub fn get_selector_action(&self, node_kind: &str) -> Option<&SelectorAction> {
        self.selectors.get(node_kind).map(|s| &s.action)
    }
}

#[derive(Clone)]
pub struct KeyContent {
    pub content: String,
}

pub fn default_parse_config_for_language(language: Language) -> ParseConfig {
    match language {
        Language::Go => {
            let mut config = ParseConfig::new(language, Indentation::Tabs);
            config.add_selector(Selector::new("source_file", SelectorAction::SelectOnly));
            config.add_selector(Selector::new(
                "import_declaration",
                SelectorAction::CaptureAll,
            ));
            config.add_selector(Selector::new(
                "function_declaration",
                SelectorAction::CaptureWithoutBlock,
            ));
            config.add_selector(Selector::new(
                "method_declaration",
                SelectorAction::CaptureWithoutBlock,
            ));

            // struct and interface
            //
            // type_declaration -> [..., type_spec] -> [..., type_name], then if type_name is
            // struct_type then get all the content of the top-most type_declaration
            config.add_selector(Selector::new(
                "type_declaration",
                SelectorAction::Custom(Box::new(|node, _cursor, source_code, _parser_state| {
                    let _node_start_position = node.start_position();
                    let _node_end_position = node.end_position();
                    let type_spec = node.child(1);
                    if let Some(type_spec) = type_spec {
                        let type_name = type_spec.child(1);
                        if let Some(type_name) = type_name {
                            let type_name_kind = type_name.kind().to_string();
                            if type_name_kind == "struct_type" || type_name_kind == "interface_type"
                            {
                                let content = node.utf8_text(source_code.as_bytes()).unwrap();
                                return Ok(content.into());
                            }
                            return Ok("".into());
                        }
                    }
                    Err(ParseError::CustomActionFailed(
                        "type_declaration custom action failed".into(),
                    ))
                })),
            ));
            config
        }
        Language::Rust => {
            let mut config = ParseConfig::new(language, Indentation::Spaces(4));
            config.add_selector(Selector::new("source_file", SelectorAction::SelectOnly));
            config.add_selector(Selector::new("use_declaration", SelectorAction::CaptureAll));
            config.add_selector(Selector::new("struct_item", SelectorAction::CaptureAll));
            config.add_selector(Selector::new("enum_item", SelectorAction::CaptureAll));
            config.add_selector(Selector::new("type_item", SelectorAction::CaptureAll));
            config.add_selector(Selector::new(
                "function_item",
                SelectorAction::CaptureWithoutBlock,
            ));
            config.add_selector(Selector::new(
                "function_signature_item",
                SelectorAction::CaptureWithoutBlock,
            ));
            config
        }
        Language::Python => {
            let mut config = ParseConfig::new(language, Indentation::Spaces(4));
            config.add_selector(Selector::new("module", SelectorAction::SelectOnly));
            config.add_selector(Selector::new(
                "future_import_statement",
                SelectorAction::CaptureAll,
            ));
            config.add_selector(Selector::new(
                "import_from_statement",
                SelectorAction::CaptureAll,
            ));
            config.add_selector(Selector::new(
                "import_statement",
                SelectorAction::CaptureAll,
            ));
            config.add_selector(Selector::new(
                "function_definition",
                SelectorAction::CaptureWithoutBlock,
            ));

            // class definitions are special because we want the class line, all fields, but also
            // we want all static and instance methods but elided without blocks.
            config.add_selector(Selector::new(
                "class_definition",
                SelectorAction::Custom(Box::new(
                    |node, _cursor, source_code, parser_state: &mut ParseState| {
                        let mut result = String::new();
                        let node_start = node.start_byte();
                        for i in 0..node.child_count() {
                            let child = node.child(i).unwrap();
                            let kind = child.kind();
                            if kind == "block" {
                                let end = child.start_byte();
                                let source = source_code.as_bytes()[node_start..end].to_vec();
                                result = String::from_utf8(source).unwrap();

                                parser_state.queue.push_front(QueueItem::Sentinel);
                                parser_state.queue.push_front(QueueItem::Node(child, true));
                            }
                        }

                        Ok(result)
                    },
                )),
            ));

            config
        }
        _ => todo!(),
    }
}

#[derive(Clone)]
struct Accumulator {
    content: Vec<String>,
}

impl Accumulator {
    fn new() -> Accumulator {
        Accumulator {
            content: Vec::new(),
        }
    }

    fn add_content(&mut self, content: String) {
        self.content.push(content);
    }

    fn finalize(&self) -> KeyContent {
        KeyContent {
            content: self.content.join("\n"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum QueueItem<'a> {
    Node(ts::Node<'a>, bool),
    Sentinel,
}

#[derive(Clone)]
pub struct ParseState<'a> {
    queue: VecDeque<QueueItem<'a>>,
    accumulator: Accumulator,
    result: Vec<KeyContent>,
    is_accumulating: bool,
}

impl<'a> ParseState<'a> {
    fn update_content(&mut self, content: String) {
        if self.is_accumulating {
            self.accumulator.add_content(content);
        } else {
            self.result.push(KeyContent { content });
        }
    }

    fn finalize_accumulator(&mut self) {
        let content = self.accumulator.finalize();
        self.result.push(content);
        self.accumulator = Accumulator::new();
    }
}

pub fn parse(source_code: &str, config: &ParseConfig) -> ParseResult<Vec<KeyContent>> {
    let tree = to_tree(source_code, &config.language_config).unwrap();
    let root_node = tree.root_node();
    let cursor = &mut root_node.walk();

    let mut state = ParseState {
        queue: VecDeque::new(),
        accumulator: Accumulator::new(),
        result: Vec::new(),
        is_accumulating: false,
    };
    state.queue.push_back(QueueItem::Node(root_node, false));

    loop {
        if state.queue.is_empty() {
            break;
        }
        let queue_item = state.queue.pop_front().unwrap();

        let node = match queue_item {
            QueueItem::Node(node, is_accumulating) => {
                state.is_accumulating = is_accumulating;
                node
            }
            QueueItem::Sentinel => {
                state.finalize_accumulator();
                continue;
            }
        };
        let node_kind = node.kind();
        println!("node_kind: {}", node_kind);

        // if there is no selector action, continue
        let selector_action = config.get_selector_action(node_kind);
        if selector_action.is_none() {
            continue;
        }
        let selector_action = selector_action.unwrap();

        match selector_action {
            SelectorAction::SelectOnly => {
                for child in node.children(cursor) {
                    state.queue.push_back(QueueItem::Node(child, false));
                }
            }
            SelectorAction::CaptureWithoutBlock => {
                let content = block_like_to_string(node, cursor, source_code, config);
                state.update_content(content);
            }
            SelectorAction::CaptureAll => {
                let content = node
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();
                state.update_content(content);
            }
            SelectorAction::Custom(action) => {
                action(node, cursor, source_code, &mut state)?;
            }
        }
    }

    Ok(state.result)
}

fn block_like_to_string<'a>(
    node: ts::Node<'a>,
    cursor: &mut ts::TreeCursor<'a>,
    source_code: &str,
    config: &ParseConfig,
) -> String {
    let capacity_guess = node.byte_range().len();
    let mut result = String::with_capacity(capacity_guess);
    for child in node.children(cursor) {
        if child.kind() == "block" {
            result.push_str(" {\n");
            result.push_str(&config.indent_value);
            result.push_str("// ...\n}");
        } else {
            if child.kind() != "parameter_list"
                && child.kind() != "func"
                && child.kind() != "type_parameters"
                && child.kind() != "parameters"
            {
                result.push(' ');
            }
            result.push_str(child.utf8_text(source_code.as_bytes()).unwrap());
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn test_parse_go() {
        let source_code = r#"
package test

import (
	"context"
)

type SetupConfig struct {
	usersTableName               repository.UsersTableName
	usernamesTableName           repository.UsernamesTableName
	emailsTableName              repository.EmailsTableName
	passwordResetTokensTableName repository.PasswordResetTokensTableName
	siteName                     string
}


func Setup(t *testing.T, setupConfig *SetupConfig) (*SetupFixture, error) {
    return nil, nil

"#
        .trim();
        let config = default_parse_config_for_language(Language::Go);
        let result = parse(source_code, &config).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(
            result[0].content,
            r#"import (
	"context"
)"#
        );
        assert_eq!(
            result[1].content,
            r#"type SetupConfig struct {
	usersTableName               repository.UsersTableName
	usernamesTableName           repository.UsernamesTableName
	emailsTableName              repository.EmailsTableName
	passwordResetTokensTableName repository.PasswordResetTokensTableName
	siteName                     string
}"#
        );
        assert_eq!(
            result[2].content,
            r#"func Setup(t *testing.T, setupConfig *SetupConfig)(*SetupFixture, error) {
	// ...
}"#
        );
    }

    #[test]
    fn test_parse_rust() {
        let source_code = r#"
use std::collections::HashMap;

pub struct Point {
    x: f64,
    y: f64,
}

pub enum Shape {
    Circle(Point, f64),
    Rectangle(Point, Point),
}

pub type PointMap = HashMap<String, Point>;

pub fn distance(p1: &Point, p2: &Point) -> f64 {
    // ...
}

pub fn area(shape: &Shape) -> f64 {
    // ...
}
"#
        .trim();
        let config = default_parse_config_for_language(Language::Rust);
        let result = parse(source_code, &config).unwrap();
        assert_eq!(result.len(), 6);
        assert_eq!(result[0].content, "use std::collections::HashMap;");
        assert_eq!(
            result[1].content,
            r#"pub struct Point {
    x: f64,
    y: f64,
}"#
        );
        assert_eq!(
            result[2].content,
            r#"pub enum Shape {
    Circle(Point, f64),
    Rectangle(Point, Point),
}"#
        );
        assert_eq!(
            result[3].content,
            "pub type PointMap = HashMap<String, Point>;"
        );
        assert_eq!(
            result[4].content,
            r#"pub fn distance(p1: &Point, p2: &Point) -> f64 {
    // ...
}"#
        );
        assert_eq!(
            result[5].content,
            r#"pub fn area(shape: &Shape) -> f64 {
    // ...
}"#
        );
    }

    #[test]
    fn test_parse_python() {
        let source_code = r#"
import math
from typing import List, Tuple

class MyClass:
    """A simple example class"""
    i = 12345
    
    def f(self):
        return 'hello world'

def f():
    return 'hello world'
"#;
        let config = default_parse_config_for_language(Language::Python);
        let result = parse(source_code, &config).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].content, "import math");
        assert_eq!(result[1].content, "from typing import List, Tuple");
        assert_eq!(
            result[2].content,
            r#"class MyClass:
    """A simple example class"""
    i = 12345

    def f(self):
        // ...

def f():
    // ...
"#
        );
    }
}
