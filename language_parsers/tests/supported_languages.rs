use language_parsers::{default_parse_config_for_language, parse, Language};

#[test]
fn parses_javascript_digest_content() {
    let source_code = r#"
import fs from "node:fs";

export function render(value) {
  return value + 1;
}
"#
    .trim();

    let result = parse(
        source_code,
        &default_parse_config_for_language(Language::JavaScript),
    )
    .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].content, r#"import fs from "node:fs";"#);
    assert_eq!(result[1].content, "function render(value) {\n  // ...\n}");
}

#[test]
fn parses_typescript_and_tsx_digest_content() {
    let typescript_source = r#"
export interface User {
  id: string;
}

export type UserId = User["id"];

export function loadUser(id: UserId): Promise<User> {
  return Promise.resolve({ id });
}
"#
    .trim();
    let tsx_source = r#"
export function View(): JSX.Element {
  return <main>Hello</main>;
}
"#
    .trim();

    let typescript_result = parse(
        typescript_source,
        &default_parse_config_for_language(Language::TypeScript),
    )
    .unwrap();
    let tsx_result = parse(
        tsx_source,
        &default_parse_config_for_language(Language::Tsx),
    )
    .unwrap();

    assert_eq!(typescript_result.len(), 3);
    assert_eq!(
        typescript_result[0].content,
        "interface User {\n  id: string;\n}"
    );
    assert_eq!(typescript_result[1].content, r#"type UserId = User["id"];"#);
    assert_eq!(
        typescript_result[2].content,
        "function loadUser(id: UserId): Promise<User> {\n  // ...\n}"
    );
    assert_eq!(tsx_result.len(), 1);
    assert_eq!(
        tsx_result[0].content,
        "function View(): JSX.Element {\n  // ...\n}"
    );
}

#[test]
fn parses_json_as_a_single_digest_unit() {
    let source_code = r#"
{
  "name": "code-digest",
  "enabled": true
}
"#
    .trim();

    let result = parse(
        source_code,
        &default_parse_config_for_language(Language::Json),
    )
    .unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].content,
        "{\n  \"name\": \"code-digest\",\n  \"enabled\": true\n}"
    );
}
