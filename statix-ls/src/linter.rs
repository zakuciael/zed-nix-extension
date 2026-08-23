use std::collections::HashMap;

use tower_lsp::lsp_types::*;

use crate::output::*;
use crate::settings::Settings;

pub async fn run(
    uri: &Url,
    settings: &Settings,
) -> Result<(Vec<Diagnostic>, Vec<CodeAction>), String> {
    let path = uri
        .to_file_path()
        .map_err(|_| format!("invalid file URI: {uri}"))?;

    let binary = settings.binary.as_deref().unwrap_or("statix");

    let mut cmd = tokio::process::Command::new(binary);
    cmd.arg("check").arg(&path).arg("-o").arg("json");

    if let Some(config) = &settings.config {
        cmd.arg("--config").arg(config);
    }

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((vec![], vec![])),
        Err(e) => return Err(format!("failed to run statix: {e}")),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.trim().is_empty() {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.trim().is_empty() {
                let stderr_lower = stderr.to_lowercase();
                if stderr_lower.contains("json")
                    || stderr_lower.contains("invalid value")
                    || stderr_lower.contains("unrecognized")
                {
                    return Err(
                        "statix does not support JSON output. Ensure statix is compiled with \
                         --all-features (e.g., install from nixpkgs)."
                            .to_string(),
                    );
                }
                return Err(format!("statix failed: {}", stderr.trim()));
            }
        }
        return Ok((vec![], vec![]));
    }

    parse_output(uri, &stdout)
}

fn parse_output(uri: &Url, stdout: &str) -> Result<(Vec<Diagnostic>, Vec<CodeAction>), String> {
    let mut diagnostics = Vec::new();
    let mut actions = Vec::new();

    let stream = serde_json::Deserializer::from_str(stdout).into_iter::<StatixOutput>();
    for item in stream {
        let output = item.map_err(|e| {
            format!(
                "failed to parse statix JSON output: {e}. \
                 Ensure statix is compiled with --all-features (e.g., install from nixpkgs)."
            )
        })?;

        for report in &output.report {
            let severity = match report.severity {
                StatixSeverity::Warn => DiagnosticSeverity::WARNING,
                StatixSeverity::Error => DiagnosticSeverity::ERROR,
                StatixSeverity::Hint => DiagnosticSeverity::HINT,
            };

            for diag in &report.diagnostics {
                let diag_range = span_to_range(&diag.at);

                diagnostics.push(Diagnostic {
                    range: diag_range,
                    severity: Some(severity),
                    code: Some(NumberOrString::Number(report.code as i32)),
                    source: Some("statix".to_string()),
                    message: format!("{}: {}", report.note, diag.message),
                    ..Default::default()
                });

                if let Some(suggestion) = &diag.suggestion {
                    let mut changes = HashMap::new();
                    changes.insert(
                        uri.clone(),
                        vec![TextEdit {
                            range: span_to_range(&suggestion.at),
                            new_text: suggestion.fix.clone(),
                        }],
                    );
                    actions.push(CodeAction {
                        title: format!("statix: {}", diag.message),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![Diagnostic {
                            range: diag_range,
                            ..Default::default()
                        }]),
                        edit: Some(WorkspaceEdit {
                            changes: Some(changes),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
            }
        }
    }

    Ok((diagnostics, actions))
}

pub fn span_to_range(span: &StatixSpan) -> Range {
    Range {
        start: Position {
            line: span.from.line.saturating_sub(1),
            character: span.from.column.saturating_sub(1),
        },
        end: Position {
            line: span.to.line.saturating_sub(1),
            character: span.to.column.saturating_sub(1),
        },
    }
}
