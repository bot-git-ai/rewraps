//! A comment's shape is preserved, whatever is inside it.
//!
//! These are the cases that were broken by re-classifying each line's *stripped*
//! text as Markdown: a `#` marker stripped from one line and then a `-` list
//! item collapsed both into one paragraph, the `#` of the next line ended up in
//! the middle of a sentence, and fenced code inside a comment was re-flowed.
//!
//! The rule these enforce: a comment is a sequence of *paragraphs* separated by
//! blank lines, and each paragraph is wrapped as text. Nothing inside a comment
//! is re-parsed as block Markdown, because the block structure of a comment is
//! its markers, not its punctuation.

use rewrap_core::{File, Settings};

fn wrap(language: &str, column: i32, input: &[&str]) -> Vec<String> {
    let file = File {
        language: language.into(),
        path: String::new(),
    };
    let settings = Settings {
        column,
        tab_width: 4,
    };
    let edit = rewrap_core::rewrap(&file, &settings, &[], input);
    if edit.is_empty() {
        input.iter().map(|line| (*line).to_string()).collect()
    } else {
        edit.lines
    }
}

#[test]
fn a_markdown_list_in_a_comment_keeps_its_markers() {
    let out = wrap(
        "python",
        30,
        &["# - alpha beta gamma delta epsilon zeta", "# - second item"],
    );
    assert_eq!(
        out,
        vec![
            "# - alpha beta gamma delta",
            "#   epsilon zeta",
            "# - second item"
        ],
        "each list item is its own paragraph"
    );
}

#[test]
fn two_paragraphs_in_a_comment_both_keep_their_text() {
    let out = wrap(
        "python",
        20,
        &[
            "# alpha beta gamma delta",
            "#",
            "# epsilon zeta eta theta iota kappa",
        ],
    );
    assert_eq!(
        out,
        vec![
            "# alpha beta gamma",
            "# delta",
            "#",
            "# epsilon zeta eta",
            "# theta iota kappa"
        ]
    );
}

#[test]
fn fenced_code_inside_a_comment_is_left_alone() {
    let input = [
        "# ```",
        "# alpha beta gamma delta epsilon zeta",
        "# ```",
        "# prose after it",
    ];
    let out = wrap("python", 30, &input);
    // The fence itself is frozen; the code between two fences is frozen with it.
    assert_eq!(out[0], "# ```");
    assert_eq!(
        out,
        vec![
            "# ```",
            "# alpha beta gamma delta epsilon zeta",
            "# ```",
            "# prose after it"
        ],
        "a fenced block inside a comment must not be re-flowed"
    );
    assert!(
        out.iter()
            .any(|line| line.contains("alpha beta gamma delta epsilon zeta")),
        "the code line was re-flowed: {out:?}"
    );
}

#[test]
fn a_quoted_line_inside_a_comment_keeps_its_marker() {
    let out = wrap("python", 30, &["# > alpha beta gamma delta epsilon zeta"]);
    assert_eq!(out[0], "# > alpha beta gamma delta");
    assert_eq!(out[1], "#   epsilon zeta", "aligned under the quoted text");
}

#[test]
fn a_hanging_indent_in_a_comment_is_kept() {
    // The shape a definition list or a numbered explanation takes: the second
    // line is indented under the first, and wrapping must not pull it back to
    // the marker's column.
    // The hanging indent is the *second* line's, and the wrapper keeps it.
    let input = [
        "# 1. alpha beta gamma delta epsilon",
        "#    zeta eta theta iota kappa",
    ];
    let out = wrap("python", 30, &input);
    assert_eq!(
        out,
        vec![
            "# 1. alpha beta gamma delta",
            "#    epsilon zeta eta theta",
            "#    iota kappa"
        ],
        "the hanging indent was lost: {out:?}"
    );
}
#[test]
fn a_hanging_indent_inside_a_docstring_is_kept() {
    let input = [
        "def f():",
        "    \"\"\"",
        "    2.  alpha beta gamma delta epsilon zeta eta theta iota kappa",
        "            hanging line that is long enough to wrap somewhere else",
        "    \"\"\"",
    ];
    let out = wrap("python", 40, &input);
    assert_eq!(out[0], "def f():");
    assert_eq!(out[1], "    \"\"\"", "the opening quotes are untouched");
    assert_eq!(out.last().map(String::as_str), Some("    \"\"\""));
    let body: Vec<&String> = out[2..out.len() - 1].iter().collect();
    // The wrapped continuation of the hanging line keeps the hanging indent,
    // which is the whole reason a docstring's own indentation is preserved.
    assert!(
        body.iter()
            .any(|line| line.starts_with("            hanging")),
        "the hanging indent was lost: {body:?}"
    );
    assert!(
        body.iter().all(|line| line.len() <= 40),
        "a body line is over the column: {body:?}"
    );
    assert!(
        body.iter().all(|line| line.len() > 4),
        "a body line lost its indentation: {body:?}"
    );
}
