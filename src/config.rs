use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub license: String,

    pub authors: Vec<String>,
    pub paths: ProjectPaths,
}

#[derive(Deserialize, Serialize)]
pub struct ProjectPaths {
    pub original: String,
    pub changes: String,
    pub revise: String,
}
