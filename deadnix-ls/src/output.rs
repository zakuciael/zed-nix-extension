use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeadnixOutput {
    pub results: Vec<DeadnixResult>,
}

#[derive(Deserialize)]
pub struct DeadnixResult {
    pub message: String,
    pub line: u32,
    pub column: u32,
    #[serde(rename = "endColumn")]
    pub end_column: u32,
}
