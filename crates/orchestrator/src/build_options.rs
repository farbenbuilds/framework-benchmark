use crate::error::OrchestratorError;

pub(crate) fn docker_build_args(
    dockerfile: &str,
    image: &str,
) -> Result<Vec<String>, OrchestratorError> {
    let mut args = vec![
        "build".into(),
        "--file".into(),
        dockerfile.into(),
        "--tag".into(),
        image.into(),
    ];
    append_enabled_option(&mut args, "BENCHMARK_IMAGE_PULL", "--pull")?;
    append_enabled_option(&mut args, "BENCHMARK_NO_CACHE", "--no-cache")?;
    args.push(".".into());
    Ok(args)
}

fn append_enabled_option(
    args: &mut Vec<String>,
    variable: &str,
    option: &str,
) -> Result<(), OrchestratorError> {
    let value = std::env::var(variable).ok();
    if parse_build_option(variable, value.as_deref())? {
        args.push(option.into());
    }
    Ok(())
}

fn parse_build_option(variable: &str, value: Option<&str>) -> Result<bool, OrchestratorError> {
    match value {
        Some("1") => Ok(true),
        Some("0") | None => Ok(false),
        Some(value) => Err(OrchestratorError::Process(format!(
            "{variable} must be 0 or 1, got {value}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_build_option;

    #[test]
    fn parse_build_option_rejects_invalid_values() {
        assert!(parse_build_option("BENCHMARK_NO_CACHE", Some("1")).unwrap());
        assert!(!parse_build_option("BENCHMARK_NO_CACHE", Some("0")).unwrap());
        assert!(!parse_build_option("BENCHMARK_NO_CACHE", None).unwrap());
        assert!(parse_build_option("BENCHMARK_NO_CACHE", Some("yes")).is_err());
    }
}
