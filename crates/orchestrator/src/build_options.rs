use crate::error::OrchestratorError;
use std::collections::HashMap;

pub(crate) fn docker_build_args(
    dockerfile: &str,
    image: &str,
    build_args: &HashMap<String, String>,
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
    let mut build_args: Vec<_> = build_args.iter().collect();
    build_args.sort_unstable_by_key(|(key, _)| *key);
    for (key, value) in build_args {
        args.extend(["--build-arg".into(), format!("{key}={value}")]);
    }
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
    use super::{docker_build_args, parse_build_option};
    use std::collections::HashMap;

    #[test]
    fn parse_build_option_rejects_invalid_values() {
        assert!(parse_build_option("BENCHMARK_NO_CACHE", Some("1")).unwrap());
        assert!(!parse_build_option("BENCHMARK_NO_CACHE", Some("0")).unwrap());
        assert!(!parse_build_option("BENCHMARK_NO_CACHE", None).unwrap());
        assert!(parse_build_option("BENCHMARK_NO_CACHE", Some("yes")).is_err());
    }

    #[test]
    fn manifest_build_arguments_are_sorted_and_forwarded() {
        let args = HashMap::from([("B_ARG".into(), "2".into()), ("A_ARG".into(), "1".into())]);
        let command = docker_build_args("Dockerfile.glibc", "image:tag", &args).unwrap();
        let rendered = command.join(" ");
        assert!(rendered.contains("--build-arg A_ARG=1 --build-arg B_ARG=2"));
    }
}
