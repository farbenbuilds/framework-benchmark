use common::result::LibcVariant;
use orchestrator::database::{DEFAULT_DATABASE_URL, start_database, stop_database};
use orchestrator::discovery::{discover_frameworks, select_frameworks};
use orchestrator::error::OrchestratorError;
use orchestrator::runner::{BenchmarkSessionConfig, execute_framework_session};
use reporter::update_readme_leaderboard;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), OrchestratorError> {
    let args: Vec<String> = env::args().collect();
    let frameworks_dir = args.get(1).map_or("frameworks", String::as_str);
    let output_dir = args.get(2).map_or("results", String::as_str);

    println!("==> Farben Framework Benchmark Orchestrator v0.1.0");
    println!("==> Scanning framework registry at: {frameworks_dir}");

    let frameworks = select_frameworks(
        discover_frameworks(Path::new(frameworks_dir))?,
        &env::var("BENCHMARK_FRAMEWORK_FILTER").unwrap_or_else(|_| "all".into()),
    )?;
    if frameworks.is_empty() {
        println!("==> No framework manifests found in {frameworks_dir}.");
        println!("==> Framework discovery completed without targets.");
        return Ok(());
    }

    println!("==> Discovered {} framework targets.", frameworks.len());
    let libcs = selected_libcs()?;
    let has_database_workloads = frameworks.iter().any(|(_, manifest)| {
        let capabilities = &manifest.capabilities;
        capabilities.db_single
            || capabilities.db_multi
            || capabilities.fortunes
            || capabilities.updates
    });
    let database = if has_database_workloads {
        start_database(Path::new("."))?
    } else {
        None
    };
    let config = BenchmarkSessionConfig {
        output_dir: output_dir.to_string(),
        readme_path: None,
        database_url: env::var("DATABASE_URL")
            .ok()
            .or_else(|| database.as_ref().map(|_| DEFAULT_DATABASE_URL.into())),
        manage_container: env::var("BENCHMARK_MANAGE_CONTAINER").map_or(true, |value| value != "0"),
        ..BenchmarkSessionConfig::default()
    };

    let execution = async {
        let mut results = Vec::new();
        let mut failures = 0_u32;
        for (dir, manifest) in frameworks {
            println!(
                "==> Running target: {} [{}] from {}",
                manifest.framework.name,
                manifest.framework.category,
                dir.display()
            );
            for libc in &libcs {
                let config = BenchmarkSessionConfig {
                    libc: *libc,
                    ..config.clone()
                };
                match execute_framework_session(&dir, &manifest, &config).await {
                    Ok(res) => {
                        let rps = res.benchmarks.first().map_or(0.0, |b| b.requests_per_sec);
                        println!(
                            "==> Benchmark completed for {}: {:.2} req/s across {} workloads",
                            manifest.framework.name,
                            rps,
                            res.benchmarks.len()
                        );
                        results.push(res);
                    }
                    Err(e) => {
                        eprintln!("==> Failed benchmark for {}: {e}", manifest.framework.name);
                        failures += 1;
                    }
                }
            }
        }
        if failures > 0 {
            return Err(OrchestratorError::Process(format!(
                "{failures} benchmark target(s) failed"
            )));
        }
        if !results.is_empty() {
            update_readme_leaderboard(Path::new("README.md"), &results)?;
        }
        Ok(())
    }
    .await;
    let cleanup = match database.as_ref() {
        Some(session) => stop_database(session),
        None => Ok(()),
    };
    match (execution, cleanup) {
        (Err(error), _) | (Ok(()), Err(error)) => return Err(error),
        (Ok(()), Ok(())) => {}
    }

    println!("==> Benchmark orchestration finished.");
    Ok(())
}

fn selected_libcs() -> Result<Vec<LibcVariant>, OrchestratorError> {
    let selected = env::var("BENCHMARK_LIBC").unwrap_or_else(|_| "all".into());
    match selected.as_str() {
        "all" => Ok(vec![LibcVariant::Glibc, LibcVariant::Musl]),
        "glibc" => Ok(vec![LibcVariant::Glibc]),
        "musl" => Ok(vec![LibcVariant::Musl]),
        value => Err(OrchestratorError::Process(format!(
            "invalid BENCHMARK_LIBC: {value}"
        ))),
    }
}
