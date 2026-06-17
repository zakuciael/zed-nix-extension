use tower_lsp::lsp_types::*;

use crate::output::DeadnixOutput;
use crate::settings::Settings;

pub async fn run(uri: &Url, settings: &Settings) -> Result<Vec<Diagnostic>, String> {
    let path = uri
        .to_file_path()
        .map_err(|_| format!("invalid file URI: {uri}"))?;

    let binary = settings.binary.as_deref().unwrap_or("deadnix");

    let mut cmd = tokio::process::Command::new(binary);
    cmd.arg("--output-format").arg("json");

    if settings.no_lambda_arg {
        cmd.arg("--no-lambda-arg");
    }
    if settings.no_lambda_pattern_names {
        cmd.arg("--no-lambda-pattern-names");
    }
    if settings.no_underscore {
        cmd.arg("--no-underscore");
    }

    cmd.arg(&path);

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(format!("failed to run deadnix: {e}")),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.trim().is_empty() {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.trim().is_empty() {
                return Err(format!("deadnix failed: {}", stderr.trim()));
            }
        }
        return Ok(vec![]);
    }

    let parsed: DeadnixOutput = serde_json::from_str(&stdout)
        .map_err(|e| format!("failed to parse deadnix output: {e}"))?;

    Ok(parsed
        .results
        .into_iter()
        .map(|r| Diagnostic {
            range: Range {
                start: Position {
                    line: r.line.saturating_sub(1),
                    character: r.column.saturating_sub(1),
                },
                end: Position {
                    line: r.line.saturating_sub(1),
                    character: r.end_column.saturating_sub(1),
                },
            },
            severity: Some(DiagnosticSeverity::WARNING),
            source: Some("deadnix".to_string()),
            message: r.message,
            ..Default::default()
        })
        .collect())
}
