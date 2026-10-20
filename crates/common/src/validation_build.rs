use crate::error::CommonError;
use crate::manifest::{FrameworkManifest, NixBuildConfig};
use crate::validation::{valid_env_key, valid_text};
use std::path::{Component, Path};

const MAX_PATH_BYTES: usize = 256;
const MAX_ENV_BYTES: usize = 1024;

pub(crate) fn validate_build(manifest: &FrameworkManifest) -> Result<(), CommonError> {
    let build = &manifest.build;
    if build.engine != "docker" && build.engine != "nix" {
        return Err(CommonError::Validation(
            "build engine must be 'docker' or 'nix'".into(),
        ));
    }
    if build.engine == "docker" {
        validate_docker(manifest)?;
    }
    if build.engine == "nix" {
        let Some(nix) = &build.nix else {
            return Err(CommonError::Validation(
                "nix engine requires nix config".into(),
            ));
        };
        validate_nix(nix)?;
    }
    Ok(())
}

fn validate_docker(manifest: &FrameworkManifest) -> Result<(), CommonError> {
    let Some(docker) = &manifest.build.docker else {
        return Err(CommonError::Validation(
            "docker engine requires docker config".into(),
        ));
    };
    if docker.dockerfile_glibc.is_none() || docker.dockerfile_musl.is_none() {
        return Err(CommonError::Validation(
            "docker engine requires glibc and musl Dockerfiles".into(),
        ));
    }
    for path in [&docker.dockerfile_glibc, &docker.dockerfile_musl]
        .into_iter()
        .flatten()
    {
        if !valid_relative_path(path) {
            return Err(CommonError::Validation(
                "dockerfile path must be a relative file name".into(),
            ));
        }
    }
    for (key, value) in &docker.build_args {
        if !valid_env_key(key) || !valid_text(value, MAX_ENV_BYTES) {
            return Err(CommonError::Validation(format!(
                "docker build argument is invalid: {key}"
            )));
        }
    }
    Ok(())
}

fn validate_nix(nix: &NixBuildConfig) -> Result<(), CommonError> {
    if nix.flake_attr_glibc.is_none() || nix.flake_attr_musl.is_none() {
        return Err(CommonError::Validation(
            "nix engine requires glibc and musl flake attributes".into(),
        ));
    }
    for attr in [&nix.flake_attr_glibc, &nix.flake_attr_musl]
        .into_iter()
        .flatten()
    {
        if !valid_nix_attr(attr) {
            return Err(CommonError::Validation(
                "nix flake attribute is invalid".into(),
            ));
        }
    }
    Ok(())
}

fn valid_relative_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PATH_BYTES
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn valid_nix_attr(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PATH_BYTES
        && !value.contains("..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
}
