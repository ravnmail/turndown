turndown
========

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/turndown.svg)](https://crates.io/crates/turndown)
[![Rust](https://img.shields.io/badge/rust-1.65.0%2B-blue.svg?maxAge=3600)](https://github.com/mwallner/turndown)

An opinionated Rust port of [Turndown.js](https://github.com/mixmark-io/turndown), 
a robust HTML to Markdown converter. This crate provides a fast, reliable way to 
transform HTML documents into clean, readable Markdown format.

## Motivation

While there are several HTML to Markdown converters available in Rust, many lack
the flexibility and configurability needed to handle the complex conversion of 
varied email HTML content. This version of `turndown` aims to fill that gap by
offering a highly customizable conversion process, allowing users to tailor the
output to their specific needs.

## Features

- Drop-in HTML to Markdown converter with sensible defaults for email content.
- Highly configurable conversion rules and formatting options.
- Support for multiple Markdown styles:
  - Heading styles: ATX (`# Heading`) or Setext (`Heading\n=======`)
  - Code block styles: Fenced (` ``` `) or Indented
  - Link styles: Inline or Reference
- Performs well on email newsletters, marketing emails and human-written emails.
- Filter tracking pixels and unnecessary elements commonly found in email HTML.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
turndown = "0.1"
```

### Basic Example

```rust
use turndown::Turndown;

let turndown = Turndown::new();
let html = "<h1>Hello</h1><p>This is a <strong>test</strong>.</p>";
let markdown = turndown.convert(html);

println!("{}", markdown);
// Output:
// # Hello
// This is a **test**.
```

### Advanced Configuration

```rust
use turndown::{Turndown, TurndownOptions, HeadingStyle, CodeBlockStyle};

let mut options = TurndownOptions::default();
options.heading_style = HeadingStyle::Setext;
options.code_block_style = CodeBlockStyle::Indented;

let turndown = Turndown::with_options(options);
let markdown = turndown.convert(html);
```

## Command-line Tool

This crate includes a CLI tool for converting HTML to Markdown from the command line:

```bash
# Convert HTML from stdin to Markdown
echo "<h1>Hello</h1>" | markdown
```

## Configuration Options

The `TurndownOptions` struct provides fine-grained control over the conversion:

- `heading_style`: Choose between ATX and Setext heading styles.
- `code_block_style`: Choose between fenced and indented code blocks.
- `link_style`: Choose between inline and reference link styles.
- `horizontal_rule`: Customize the horizontal rule character sequence.

## Architecture

The conversion process works in two main stages:

1. **Parsing**: HTML is parsed into a DOM tree using the `html5ever` parser.
2. **Conversion**: The DOM tree is traversed and converted to Markdown using a rule-based system.

The rule-based system allows for customization and extension of the conversion logic.

## License

Licensed under either of:

 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
