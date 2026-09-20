//! Pretty-format UTF-8 BibTeX files in-place, reporting parse errors with source annotations.
//!
//! Run with
//!
//! ```sh
//! cargo run --example format --features entry -- input.bib ...
//! ```
//!
//! Note that empty `@string{}` entries and TeX-style comments `%...` are deleted from the input.

use std::{env, fs, path::Path, process::ExitCode};

use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet, renderer::DecorStyle};
use serde_bibtex::{Error, entry::RawBibliography, from_str, to_string};

/// Format a file at the given input path in-place.
fn format_file(input: &Path) -> Result<(), String> {
    let source = fs::read_to_string(input)
        .map_err(|error| format!("could not read {}: {error}", input.display()))?;

    // deserialize the source into a raw bibliography, which does not attempt to expand macros
    // or collapse concatenations
    let bibliography: RawBibliography<'_> =
        from_str(&source).map_err(|error| diagnostic(input, &source, &error))?;
    let formatted = to_string(&bibliography).expect("failed to serialize valid bibliography");
    fs::write(input, formatted)
        .map_err(|error| format!("could not write {}: {error}", input.display()))
}

/// Produce a diagnostic for the given path and error.
fn diagnostic(path: &Path, source: &str, error: &Error) -> String {
    let title = error.to_string();
    let path = path.to_string_lossy();
    let mut report = Group::with_title(Level::ERROR.primary_title(&title));
    if let Some(span) = error.span() {
        report = report.element(
            Snippet::source(source)
                .path(path.as_ref())
                .annotation(AnnotationKind::Primary.span(span)),
        );
    }
    Renderer::styled()
        .decor_style(DecorStyle::Unicode)
        .render(&[report])
}

#[allow(clippy::print_stderr)]
fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();

    let mut errors = Vec::new();
    for input in args {
        if let Err(error) = format_file(Path::new(&input)) {
            errors.push(error);
        }
    }
    if errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        for e in errors {
            eprintln!("{e}");
        }
        ExitCode::FAILURE
    }
}
