use orchestrator::discovery::discover_frameworks;
use orchestrator::error::OrchestratorError;
use orchestrator::runner::{BenchmarkSessionConfig, execute_framework_session};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), OrchestratorError> {
    let args: Vec<String> = env::args().collect();
    let frameworks_dir = args.get(1).map_or("frameworks", String::as_str);
    let output_dir = args.get(2).map_or("results", String::as_str);

    println!("==> Farben Framework Benchmark Orchestrator v0.1.0");
    println!("==> Scanning framework registry at: {frameworks_dir}");

    let frameworks = discover_frameworks(Path::new(frameworks_dir))?;
    if frameworks.is_empty() {
        println!("==> No framework manifests found in {frameworks_dir}.");
        println!("==> Framework discovery completed without targets.");
        return Ok(());
    }

    println!("==> Discovered {} framework targets.", frameworks.len());
    let config = BenchmarkSessionConfig {
        output_dir: output_dir.to_string(),
        ..BenchmarkSessionConfig::default()
    };

    for (dir, manifest) in frameworks {
        println!(
            "==> Running target: {} [{}] from {}",
            manifest.framework.name,
            manifest.framework.category,
            dir.display()
        );
        match execute_framework_session(&manifest, &config).await {
            Ok(res) => {
                let rps = res.benchmarks.first().map_or(0.0, |b| b.requests_per_sec);
                println!(
                    "==> Benchmark completed for {}: {:.2} req/s across {} workloads",
                    manifest.framework.name,
                    rps,
                    res.benchmarks.len()
                );
            }
            Err(e) => eprintln!("==> Failed benchmark for {}: {e}", manifest.framework.name),
        }
    }

    println!("==> Benchmark orchestration finished.");
    Ok(())
}
