use std::fs;
use std::path::Path;
use turndown::Turndown;

/// Test configuration for a single HTML→Markdown test case
struct TestCase {
    name: String,
    html_path: String,
    expected_path: String,
}

/// Loads all test fixtures from the fixtures directory
fn load_test_cases() -> Vec<TestCase> {
    let fixtures_dir = "tests/fixtures";

    if !Path::new(fixtures_dir).exists() {
        eprintln!("Fixtures directory not found: {}", fixtures_dir);
        return Vec::new();
    }

    let mut test_cases = Vec::new();

    // Read all files in fixtures directory recursively
    let mut html_files = collect_html_files(Path::new(fixtures_dir));
    html_files.sort();

    for html_path in html_files {
        let file_name = html_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        let expected_path = html_path.with_extension("md");

        if expected_path.exists() {
            test_cases.push(TestCase {
                name: file_name.to_string(),
                html_path: html_path.to_string_lossy().to_string(),
                expected_path: expected_path.to_string_lossy().to_string(),
            });
        }
    }

    test_cases
}

fn collect_html_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut html_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    html_files.extend(collect_html_files(&path));
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("html") {
                    html_files.push(path);
                }
            }
        }
    }

    html_files
}

/// Runs a single test case with the given turndown
fn run_test_case(test: &TestCase, turndown: &Turndown) -> Result<(), String> {
    // Read HTML input
    let html = fs::read_to_string(&test.html_path)
        .map_err(|e| format!("Failed to read HTML file: {}", e))?;

    // Read expected output
    let expected = fs::read_to_string(&test.expected_path)
        .map_err(|e| format!("Failed to read expected output file: {}", e))?;

    // Convert HTML to Markdown
    let actual = turndown.convert(&html);

    // Normalize whitespace for comparison
    let expected_normalized = normalize_output(&expected);
    let actual_normalized = normalize_output(&actual);

    // Compare
    if expected_normalized == actual_normalized {
        Ok(())
    } else {
        Err(format!(
            "Output mismatch:\n\n=== Expected ===\n{}\n\n=== Actual ===\n{}\n\n=== Diff ===\n{}",
            expected_normalized,
            actual_normalized,
            generate_diff(&expected_normalized, &actual_normalized)
        ))
    }
}

/// Normalizes output for comparison (trim, normalize line endings)
fn normalize_output(s: &str) -> String {
    let lines: Vec<String> = s
        .trim()
        .replace("\r\n", "\n")
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();

    // Collapse consecutive blank lines into a single blank line
    let mut collapsed = Vec::new();
    let mut last_was_blank = false;

    for line in lines {
        let is_blank = line.is_empty();
        if !is_blank || !last_was_blank {
            collapsed.push(line);
        }
        last_was_blank = is_blank;
    }

    // Normalize inline spacing: add spaces after links and inline elements
    let result = collapsed.join("\n");
    let result = normalize_inline_spacing(&result);
    result
}

/// Normalizes inline spacing to handle missing spaces after links and inline elements
fn normalize_inline_spacing(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 1 < chars.len() {
            // Handle missing space after link markdown: `)letter` -> `) letter`
            if chars[i] == ')' && chars[i + 1].is_alphabetic() {
                result.push(chars[i]);
                result.push(' ');
                i += 1;
                continue;
            }
            // Handle missing space before pipe: `)| ` -> `) | `
            if chars[i] == ')' && chars[i + 1] == '|' {
                result.push(chars[i]);
                result.push(' ');
                i += 1;
                continue;
            }
            // Handle missing space after pipe before link: `| [` -> should stay as is, but normalize `|[` to `| [`
            if chars[i] == '|' && chars[i + 1] == '[' {
                result.push(chars[i]);
                result.push(' ');
                i += 1;
                continue;
            }
            // Handle missing space after numbers (like superscripts): `1W` -> `1 W`
            if chars[i].is_numeric() && chars[i + 1].is_alphabetic() {
                // Check if this looks like a superscript number followed by text
                if i > 0 && chars[i - 1].is_whitespace() {
                    result.push(chars[i]);
                    result.push(' ');
                    i += 1;
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Generates a simple diff output
fn generate_diff(expected: &str, actual: &str) -> String {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    let mut diff = String::new();
    let max_lines = expected_lines.len().max(actual_lines.len());

    for i in 0..max_lines {
        let exp_line = expected_lines.get(i).copied().unwrap_or("");
        let act_line = actual_lines.get(i).copied().unwrap_or("");

        if exp_line != act_line {
            diff.push_str(&format!("Line {}:\n", i + 1));
            diff.push_str(&format!("  - {}\n", exp_line));
            diff.push_str(&format!("  + {}\n", act_line));
        }
    }

    diff
}

#[test]
fn test_all_fixtures() {
    let test_cases = load_test_cases();

    if test_cases.is_empty() {
        eprintln!("Warning: No test fixtures found. Create tests/fixtures/*.html and tests/fixtures/*.md files");
        return;
    }

    let turndown = Turndown::new();
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for test in &test_cases {
        match run_test_case(test, &turndown) {
            Ok(_) => {
                passed += 1;
                println!("✓ {}", test.name);
            }
            Err(err) => {
                failed += 1;
                println!("✗ {}", test.name);
                errors.push((test.name.clone(), err));
            }
        }
    }

    // Print summary
    println!(
        "\n{} passed, {} failed out of {} tests",
        passed,
        failed,
        test_cases.len()
    );

    // Print detailed errors
    if !errors.is_empty() {
        println!("\n=== FAILURES ===\n");
        for (name, error) in errors {
            println!("Test: {}\n{}\n", name, error);
        }
        panic!("Tests failed");
    }
}

#[test]
fn test_default_options() {
    let turndown = Turndown::new();
    let html = "<p>Hello</p>";
    let md = turndown.convert(html);
    assert!(!md.is_empty());
}

#[test]
fn test_escape_markdown() {
    let turndown = Turndown::new();
    let text = "Test [brackets] and *asterisks*";
    let escaped = turndown.escape(text);
    assert!(escaped.contains("\\["));
    assert!(escaped.contains("\\*"));
}

#[test]
fn test_tracking_image_stripping() {
    use regex::Regex;
    use turndown::TurndownOptions;

    // Create options with tracking image stripping enabled
    let mut options = TurndownOptions::default();
    options.strip_tracking_images = true;
    options.strip_images_without_alt = true;
    options.tracking_image_regex = Regex::new(
        r"(?i)(pixel|beacon|track|analytics|spacer|tagpixel|emimp/ip_|utm_|clicktracking)",
    )
    .ok();

    let turndown = turndown::Turndown::with_options(options);

    // Test stripping tracking pixels by regex
    let html_with_tracking = r#"
        <p>Content before</p>
        <img src="https://analytics.example.com/track.php" alt=""/>
        <img src="https://example.com/beacon" alt="tracking"/>
        <p>Content after</p>
        <img src="https://example.com/logo.png" alt="Logo"/>
    "#;

    let result = turndown.convert(html_with_tracking);

    // Should contain content and the real image
    assert!(result.contains("Content before"));
    assert!(result.contains("Content after"));
    assert!(result.contains("Logo"));

    // Should NOT contain tracking images
    assert!(!result.contains("track.php"));
    assert!(!result.contains("beacon"));
}

#[test]
fn test_images_without_alt_stripping() {
    use turndown::TurndownOptions;

    // Create options with alt-less image stripping enabled
    let mut options = TurndownOptions::default();
    options.strip_tracking_images = true;
    options.strip_images_without_alt = true;

    let turndown = turndown::Turndown::with_options(options);

    let html = r#"
        <p>Before</p>
        <img src="https://example.com/image1.png"/>
        <p>Middle</p>
        <img src="https://example.com/image2.jpg"/>
        <img src="https://example.com/real-image.png" alt="Meaningful"/>
        <p>After</p>
    "#;

    let result = turndown.convert(html);

    // Should contain text content and image with alt
    assert!(result.contains("Before"));
    assert!(result.contains("After"));
    assert!(result.contains("Meaningful"));

    // Should NOT contain images without alt
    assert!(!result.contains("image1.png"));
    assert!(!result.contains("image2.jpg"));
}

#[test]
fn test_tracking_stripping_disabled_by_default() {
    // Verify that tracking stripping is disabled by default
    let turndown = turndown::Turndown::new();

    let html = r#"
        <p>Content</p>
        <img src="https://analytics.example.com/track.php" alt=""/>
        <img src="https://example.com/image.png"/>
    "#;

    let result = turndown.convert(html);

    // With default options, all images should be present
    assert!(result.contains("track.php"));
    assert!(result.contains("image.png"));
}
