mod tree_sitter_parse;

pub enum KeyContentType {
    Function,
    Class,
    Module,
}

pub struct KeyContent {
    pub name: String,
    pub content_type: KeyContentType,
    pub line_number: usize,
}

pub trait LanguageParser {
    fn parse(&self, source_code: &str) -> Vec<KeyContent>;
}
