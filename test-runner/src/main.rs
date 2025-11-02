/*!
Cross-language test runner for transpilation validation.

This tool orchestrates tests against multiple language implementations
to ensure correctness of transpiled code.
*/

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tonic::transport::Channel;
use tracing::{debug, info, warn};

// Generated proto code
pub mod transpile_test {
    tonic::include_proto!("transpile_test");
}

use transpile_test::transpile_test_service_client::TranspileTestServiceClient;
use transpile_test::*;

#[derive(Debug, Deserialize, Serialize)]
struct TestSuite {
    name: String,
    description: Option<String>,
    servers: TestServers,
    tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TestServers {
    python: ServerConfig,
    rust: ServerConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
struct TestCase {
    name: String,
    description: Option<String>,
    method: String,
    arguments: serde_json::Value,
    #[serde(default)]
    stateful: bool,
    #[serde(default)]
    initial_state: Option<String>,
    expected: Option<serde_json::Value>,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    python_result: Option<serde_json::Value>,
    rust_result: Option<serde_json::Value>,
    python_error: Option<String>,
    rust_error: Option<String>,
    python_time_us: Option<i64>,
    rust_time_us: Option<i64>,
    error_message: Option<String>,
}

struct TestRunner {
    python_client: TranspileTestServiceClient<Channel>,
    rust_client: TranspileTestServiceClient<Channel>,
}

impl TestRunner {
    async fn new(servers: &TestServers) -> Result<Self> {
        let python_url = format!("http://{}:{}", servers.python.host, servers.python.port);
        let rust_url = format!("http://{}:{}", servers.rust.host, servers.rust.port);

        info!("Connecting to Python server at {}", python_url);
        let python_client = TranspileTestServiceClient::connect(python_url)
            .await
            .context("Failed to connect to Python server")?;

        info!("Connecting to Rust server at {}", rust_url);
        let rust_client = TranspileTestServiceClient::connect(rust_url)
            .await
            .context("Failed to connect to Rust server")?;

        Ok(Self {
            python_client,
            rust_client,
        })
    }

    async fn run_test(&mut self, test: &TestCase) -> Result<TestResult> {
        info!("Running test: {}", test.name);

        let args_json = serde_json::to_string(&test.arguments)?;

        // Run test on Python
        let (python_result, python_error, python_time) =
            self.execute_on_python(test, &args_json).await;

        // Run test on Rust
        let (rust_result, rust_error, rust_time) = self.execute_on_rust(test, &args_json).await;

        // Compare results
        let (passed, error_message) = self.compare_results(
            &python_result,
            &rust_result,
            &python_error,
            &rust_error,
            &test.expected,
        );

        Ok(TestResult {
            name: test.name.clone(),
            passed,
            python_result,
            rust_result,
            python_error,
            rust_error,
            python_time_us: python_time,
            rust_time_us: rust_time,
            error_message,
        })
    }

    async fn execute_on_python(
        &mut self,
        test: &TestCase,
        args_json: &str,
    ) -> (Option<serde_json::Value>, Option<String>, Option<i64>) {
        let context_id = if test.stateful {
            match self
                .python_client
                .create_context(CreateContextRequest {
                    initial_state: test.initial_state.clone().unwrap_or_default(),
                })
                .await
            {
                Ok(resp) => {
                    let resp = resp.into_inner();
                    if resp.success {
                        Some(resp.context_id)
                    } else {
                        return (None, Some(resp.error), None);
                    }
                }
                Err(e) => return (None, Some(e.to_string()), None),
            }
        } else {
            None
        };

        let request = InvokeMethodRequest {
            context_id: context_id.clone().unwrap_or_default(),
            method_name: test.method.clone(),
            arguments: args_json.to_string(),
        };

        let result = match self.python_client.invoke_method(request).await {
            Ok(resp) => {
                let resp = resp.into_inner();
                if resp.success {
                    let result: Option<serde_json::Value> =
                        serde_json::from_str(&resp.result).ok();
                    let time = resp.metadata.as_ref().map(|m| m.execution_time_us);
                    (result, None, time)
                } else {
                    (None, Some(resp.error), None)
                }
            }
            Err(e) => (None, Some(e.to_string()), None),
        };

        // Cleanup context if needed
        if let Some(ctx_id) = context_id {
            let _ = self
                .python_client
                .destroy_context(DestroyContextRequest { context_id: ctx_id })
                .await;
        }

        result
    }

    async fn execute_on_rust(
        &mut self,
        test: &TestCase,
        args_json: &str,
    ) -> (Option<serde_json::Value>, Option<String>, Option<i64>) {
        let context_id = if test.stateful {
            match self
                .rust_client
                .create_context(CreateContextRequest {
                    initial_state: test.initial_state.clone().unwrap_or_default(),
                })
                .await
            {
                Ok(resp) => {
                    let resp = resp.into_inner();
                    if resp.success {
                        Some(resp.context_id)
                    } else {
                        return (None, Some(resp.error), None);
                    }
                }
                Err(e) => return (None, Some(e.to_string()), None),
            }
        } else {
            None
        };

        let request = InvokeMethodRequest {
            context_id: context_id.clone().unwrap_or_default(),
            method_name: test.method.clone(),
            arguments: args_json.to_string(),
        };

        let result = match self.rust_client.invoke_method(request).await {
            Ok(resp) => {
                let resp = resp.into_inner();
                if resp.success {
                    let result: Option<serde_json::Value> =
                        serde_json::from_str(&resp.result).ok();
                    let time = resp.metadata.as_ref().map(|m| m.execution_time_us);
                    (result, None, time)
                } else {
                    (None, Some(resp.error), None)
                }
            }
            Err(e) => (None, Some(e.to_string()), None),
        };

        // Cleanup context if needed
        if let Some(ctx_id) = context_id {
            let _ = self
                .rust_client
                .destroy_context(DestroyContextRequest { context_id: ctx_id })
                .await;
        }

        result
    }

    fn compare_results(
        &self,
        python_result: &Option<serde_json::Value>,
        rust_result: &Option<serde_json::Value>,
        python_error: &Option<String>,
        rust_error: &Option<String>,
        expected: &Option<serde_json::Value>,
    ) -> (bool, Option<String>) {
        // Both errored
        if python_error.is_some() && rust_error.is_some() {
            return (
                false,
                Some(format!(
                    "Both implementations failed:\nPython: {}\nRust: {}",
                    python_error.as_ref().unwrap(),
                    rust_error.as_ref().unwrap()
                )),
            );
        }

        // Only one errored
        if python_error.is_some() {
            return (
                false,
                Some(format!(
                    "Python failed: {}",
                    python_error.as_ref().unwrap()
                )),
            );
        }

        if rust_error.is_some() {
            return (
                false,
                Some(format!("Rust failed: {}", rust_error.as_ref().unwrap())),
            );
        }

        // Compare results
        if python_result != rust_result {
            return (
                false,
                Some(format!(
                    "Results differ:\nPython: {:?}\nRust: {:?}",
                    python_result, rust_result
                )),
            );
        }

        // Check against expected if provided
        if let Some(exp) = expected {
            if Some(exp) != python_result.as_ref() {
                return (
                    false,
                    Some(format!(
                        "Result doesn't match expected:\nExpected: {:?}\nGot: {:?}",
                        exp, python_result
                    )),
                );
            }
        }

        (true, None)
    }
}

fn print_results(suite_name: &str, results: &[TestResult]) {
    println!("\n{}", "=".repeat(80).bright_blue());
    println!("{}: {}", "Test Suite".bright_blue().bold(), suite_name);
    println!("{}", "=".repeat(80).bright_blue());

    let mut passed = 0;
    let mut failed = 0;

    for result in results {
        if result.passed {
            passed += 1;
            println!(
                "\n  {} {}",
                "✓".bright_green().bold(),
                result.name.bright_white()
            );

            if let (Some(py_time), Some(rs_time)) = (result.python_time_us, result.rust_time_us) {
                println!(
                    "    ⏱  Python: {}μs | Rust: {}μs",
                    py_time.to_string().cyan(),
                    rs_time.to_string().cyan()
                );
            }

            if let Some(ref res) = result.python_result {
                println!("    Result: {}", serde_json::to_string(res).unwrap().dimmed());
            }
        } else {
            failed += 1;
            println!(
                "\n  {} {}",
                "✗".bright_red().bold(),
                result.name.bright_white()
            );

            if let Some(ref err) = result.error_message {
                for line in err.lines() {
                    println!("    {}", line.red());
                }
            }
        }
    }

    println!("\n{}", "=".repeat(80).bright_blue());
    println!(
        "{}: {}/{} passed",
        "Summary".bright_blue().bold(),
        passed.to_string().bright_green(),
        (passed + failed).to_string().bright_white()
    );

    if failed > 0 {
        println!("  {} tests failed", failed.to_string().bright_red());
    }
    println!("{}\n", "=".repeat(80).bright_blue());
}

#[derive(Parser)]
#[command(name = "transpile-test-runner")]
#[command(about = "Run cross-language transpilation tests")]
struct Args {
    /// Path to test suite YAML file
    #[arg(short, long)]
    suite: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    // Load test suite
    info!("Loading test suite from: {}", args.suite.display());
    let suite_content = tokio::fs::read_to_string(&args.suite)
        .await
        .context("Failed to read test suite file")?;

    let suite: TestSuite =
        serde_yaml::from_str(&suite_content).context("Failed to parse test suite YAML")?;

    info!("Loaded test suite: {}", suite.name);
    if let Some(ref desc) = suite.description {
        info!("Description: {}", desc);
    }

    // Create test runner
    let mut runner = TestRunner::new(&suite.servers).await?;

    // Run all tests
    let mut results = Vec::new();
    for test in &suite.tests {
        match runner.run_test(test).await {
            Ok(result) => results.push(result),
            Err(e) => {
                warn!("Failed to run test {}: {}", test.name, e);
                results.push(TestResult {
                    name: test.name.clone(),
                    passed: false,
                    python_result: None,
                    rust_result: None,
                    python_error: None,
                    rust_error: None,
                    python_time_us: None,
                    rust_time_us: None,
                    error_message: Some(format!("Test execution failed: {}", e)),
                });
            }
        }
    }

    // Print results
    print_results(&suite.name, &results);

    // Exit with error code if any tests failed
    if results.iter().any(|r| !r.passed) {
        std::process::exit(1);
    }

    Ok(())
}
