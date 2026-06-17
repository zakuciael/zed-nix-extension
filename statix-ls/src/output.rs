use serde::Deserialize;

#[derive(Deserialize)]
pub struct StatixOutput {
    pub report: Vec<StatixReport>,
}

#[derive(Deserialize)]
pub struct StatixReport {
    pub note: String,
    pub code: u32,
    pub severity: StatixSeverity,
    pub diagnostics: Vec<StatixDiagnostic>,
}

#[derive(Deserialize)]
pub enum StatixSeverity {
    Warn,
    Error,
    Hint,
}

#[derive(Deserialize)]
pub struct StatixDiagnostic {
    pub at: StatixSpan,
    pub message: String,
    pub suggestion: Option<StatixSuggestion>,
}

#[derive(Deserialize)]
pub struct StatixSuggestion {
    pub at: StatixSpan,
    pub fix: String,
}

#[derive(Deserialize)]
pub struct StatixSpan {
    pub from: StatixPosition,
    pub to: StatixPosition,
}

#[derive(Deserialize)]
pub struct StatixPosition {
    pub line: u32,
    pub column: u32,
}
