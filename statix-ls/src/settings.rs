use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct Settings {
    pub binary: Option<String>,
    pub config: Option<String>,
}
