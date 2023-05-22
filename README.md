<h1 align="center">
  code-digest
</h1>

<p align="center">
    <a href="#overview">Overview</a> •
    <a href="#features">Features</a> •
    <a href="#install">Install</a> •
    <a href="#examples">Examples</a> •
    <a href="#usage">Usage</a> •
    <a href="#library">Library</a> •
    <a href="#cli-tool">CLI Tool</a> •
    <a href="#cross-compilation">Cross-Compilation</a> •
    <a href="#contributing">Contributing</a> •
    <a href="#license">License</a>
</p>

Welcome to the code-digest repository, a versatile library and toolset for
extracting essential information from codebases and preparing inputs for large
language models.

[![codecov](https://codecov.io/gh/asimihsan/code-digest/branch/main/graph/badge.svg?token=70NAB25YOV)](https://codecov.io/gh/asimihsan/code-digest)

## Overview

code-digest is designed to help developers and researchers analyze codebases,
extract key information, and prepare data for use with large language models.
With support for multiple programming languages and a flexible parsing system,
code-digest can be easily adapted to various use cases and requirements.

## Features

- Support for multiple programming languages, including Go, HCL, Java, Python,
  and Rust
- Flexible parsing system with customizable selectors and actions
- Efficient file system traversal with support for ignoring specific directories
- Cross-compilation to Android, iOS, and other platforms (library only)
- Command-line interface (CLI) tool for easy integration into existing workflows

## Install

```shell
brew tap asimihsan/code-digest
brew install asimihsan/code-digest/code-digest
```

## Examples

Input Rust code:

```rust
pub struct Point {
    x: f64,
    y: f64,
}

pub enum Shape {
    Circle(Point, f64),
    Rectangle(Point, Point),
}

pub fn distance(p1: &Point, p2: &Point) -> f64 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    (dx * dx + dy * dy).sqrt()
}

pub fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle(_, radius) => std::f64::consts::PI * radius * radius,
        Shape::Rectangle(p1, p2) => (p1.x - p2.x).abs() * (p1.y - p2.y).abs(),
    }
}
```

With default settings, the output key content:

```text
pub struct Point {
    x: f64,
    y: f64,
}

pub enum Shape {
    Circle(Point, f64),
    Rectangle(Point, Point),
}

pub fn distance(p1: &Point, p2: &Point) -> f64 {
    // ...
}

pub fn area(shape: &Shape) -> f64 {
    // ...
}
```

## Usage

To use `code-digest`, you can either integrate the library into your own project
or use the provided CLI tool.

### Library

The `code-digest` library provides a set of functions and structures for parsing
source code and extracting key information. To use the library, simply add it as
a dependency in your project and import the necessary modules.

For example, to parse a Rust source file and extract key content, you can use
the following code:

```rust
use code_digest::language_parsers::{default_parse_config_for_language, parse};
use code_digest::Language;

let source_code = "..."; // Your Rust source code
let config = default_parse_config_for_language(Language::Rust);
let result = parse(source_code, &config);

match result {
    Ok(key_contents) => {
        for key_content in key_contents {
            println!("{}", key_content.content);
        }
    }
    Err(error) => {
        eprintln!("Error: {}", error);
    }
}
```

### CLI Tool

The CLI tool provides a convenient way to use `code-digest` without integrating
it into your own project. To use the CLI tool, simply run it with the
appropriate arguments, specifying the directory containing the source files and
any additional directories to ignore.

For example, to analyze a Rust project and extract key content, you can run the
following command:

```sh
code-digest --directory /path/to/your/project --ignore /path/to/ignore/directory
```

## Cross-Compilation

The `code-digest` library can be cross-compiled to various platforms, including
Android, iOS, and others. This allows you to use the library in mobile
applications and other environments where cross-compilation is required. Note
that the CLI tool is not intended for cross-compilation and is provided as a
demonstration of the library's capabilities.

## Developing

### Mac release

#### Pre-requisites

1. Get your developer ID:

    ```shell
    security find-identity -v
    ```

2. Generate an app-specific password in your Apple ID account then put it into Keychain undedr the name `AC_PASSWORD`.
3. Create a bundle ID in Apple, e.g. `com.foo.baz`.
4. Use `.env.template` to create `.env` and fill in the values.
5. You need a developer certificate, see https://developer.apple.com/help/account/create-certificates/create-developer-id-certificates

#### Steps

```shell
source .env
./scripts/notarize.sh \
  --binary-path "target/aarch64-apple-darwin/release/code-digest" \
  --output-zip-path "target/aarch64-apple-darwin/release/code-digest.zip" \
  --developer-id "$DEVELOPER_ID" \
  --apple-id "$APPLE_ID" \
  --app-specific-password "$APP_SPECIFIC_PASSWORD"
```

## Contributing

Contributions to `code-digest` are welcome! If you have a feature request, bug
report, or want to contribute code, please open an issue or submit a pull
request on GitHub.

## License

This project is licensed under the terms of the Mozilla Public License 2.0
license.

The MPL-2.0 license allows you to use, share, and modify the code in this
repository for any purpose, including commercial use. All code taken from this
repository must remain open source. If you make any changes to the code, you
must also make those changes available under the same license. This is not a
viral license, meaning that it does not require you to open source any other
code you write or use in combination with this code, as long as the code from
this repository remains open source. The MPL-2.0 license also includes a patent
grant, which provides protection against patent infringement claims.

See the [LICENSE](LICENSE) file for details.
