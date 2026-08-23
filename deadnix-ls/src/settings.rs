use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct Settings {
    pub binary: Option<String>,
    #[serde(default)]
    pub no_lambda_arg: bool,
    #[serde(default)]
    pub no_lambda_pattern_names: bool,
    #[serde(default)]
    pub no_underscore: bool,
}
