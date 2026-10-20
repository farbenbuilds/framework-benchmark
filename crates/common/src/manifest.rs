use crate::error::CommonError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameworkMetadata {
    pub name: String,
    pub version: String,
    pub language: String,
    pub language_version: String,
    pub website: Option<String>,
    pub category: String,
    pub tier: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default)]
    pub plaintext: bool,
    #[serde(default)]
    pub json: bool,
    #[serde(default)]
    pub db_single: bool,
    #[serde(default)]
    pub db_multi: bool,
    #[serde(default)]
    pub fortunes: bool,
    #[serde(default)]
    pub updates: bool,
    #[serde(default)]
    pub websocket: bool,
    #[serde(default)]
    pub http2: bool,
    #[serde(default)]
    pub http3: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerBuildConfig {
    pub dockerfile_glibc: Option<String>,
    pub dockerfile_musl: Option<String>,
    #[serde(default)]
    pub build_args: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NixBuildConfig {
    pub flake_attr_glibc: Option<String>,
    pub flake_attr_musl: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildConfig {
    pub engine: String,
    pub docker: Option<DockerBuildConfig>,
    pub nix: Option<NixBuildConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub port: u16,
    pub healthcheck_path: String,
    #[serde(default)]
    pub websocket_path: Option<String>,
    #[serde(default = "default_startup_timeout")]
    pub startup_timeout_seconds: u64,
    pub shutdown_signal: Option<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

fn default_startup_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameworkManifest {
    pub framework: FrameworkMetadata,
    pub capabilities: Capabilities,
    pub build: BuildConfig,
    pub runtime: RuntimeConfig,
}

pub fn parse_manifest(toml_str: &str) -> Result<FrameworkManifest, CommonError> {
    let manifest: FrameworkManifest = toml::from_str(toml_str)?;
    crate::validation::validate_manifest(&manifest)?;
    Ok(manifest)
}
