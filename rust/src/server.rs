/*!
Rust gRPC server for transpilation testing.

This server allows executing Rust functions over gRPC with support for:
- Stateless function calls
- Stateful execution contexts
- Dynamic function registration
*/

use clap::Parser;
use parking_lot::RwLock;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{debug, error, info};
use uuid::Uuid;

// Generated proto code
pub mod transpile_test {
    tonic::include_proto!("transpile_test");
}

use transpile_test::transpile_test_service_server::{
    TranspileTestService, TranspileTestServiceServer,
};
use transpile_test::*;

mod examples;

/// Type alias for registered functions
type RegisteredFunction =
    Arc<dyn Fn(&ExecutionContext, JsonValue) -> Result<JsonValue, String> + Send + Sync>;

/// Execution context for stateful function calls
#[derive(Clone)]
pub struct ExecutionContext {
    context_id: String,
    state: Arc<RwLock<HashMap<String, JsonValue>>>,
}

impl ExecutionContext {
    fn new(context_id: String, initial_state: Option<String>) -> Self {
        let state = if let Some(init) = initial_state {
            serde_json::from_str(&init).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            context_id,
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn get_state(&self, key: &str) -> Option<JsonValue> {
        self.state.read().get(key).cloned()
    }

    pub fn set_state(&self, key: String, value: JsonValue) {
        self.state.write().insert(key, value);
    }

    pub fn get_all_state(&self) -> HashMap<String, JsonValue> {
        self.state.read().clone()
    }
}

/// Metadata about a registered function
#[derive(Clone)]
struct FunctionMetadata {
    description: String,
    is_stateful: bool,
    parameter_types: Vec<String>,
    return_type: String,
}

/// Service implementation
pub struct TranspileTestServer {
    contexts: Arc<RwLock<HashMap<String, ExecutionContext>>>,
    methods: Arc<RwLock<HashMap<String, RegisteredFunction>>>,
    metadata: Arc<RwLock<HashMap<String, FunctionMetadata>>>,
}

impl TranspileTestServer {
    pub fn new() -> Self {
        info!("Initializing Rust gRPC server");
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            methods: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a function that can be invoked via gRPC
    pub fn register_function<F>(
        &self,
        name: impl Into<String>,
        func: F,
        description: impl Into<String>,
        is_stateful: bool,
        parameter_types: Vec<String>,
        return_type: impl Into<String>,
    ) where
        F: Fn(&ExecutionContext, JsonValue) -> Result<JsonValue, String> + Send + Sync + 'static,
    {
        let name = name.into();
        let description = description.into();
        let return_type = return_type.into();

        self.methods.write().insert(name.clone(), Arc::new(func));
        self.metadata.write().insert(
            name.clone(),
            FunctionMetadata {
                description,
                is_stateful,
                parameter_types,
                return_type,
            },
        );

        info!("Registered function: {}", name);
    }
}

impl Default for TranspileTestServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl TranspileTestService for TranspileTestServer {
    async fn create_context(
        &self,
        request: Request<CreateContextRequest>,
    ) -> Result<Response<CreateContextResponse>, Status> {
        let req = request.into_inner();
        let context_id = Uuid::new_v4().to_string();

        let initial_state = if req.initial_state.is_empty() {
            None
        } else {
            Some(req.initial_state)
        };

        let context = ExecutionContext::new(context_id.clone(), initial_state);
        self.contexts.write().insert(context_id.clone(), context);

        info!("Created context: {}", context_id);

        Ok(Response::new(CreateContextResponse {
            context_id,
            success: true,
            error: String::new(),
        }))
    }

    async fn invoke_method(
        &self,
        request: Request<InvokeMethodRequest>,
    ) -> Result<Response<InvokeMethodResponse>, Status> {
        let req = request.into_inner();
        let start = Instant::now();

        // Get the function
        let func = {
            let methods = self.methods.read();
            match methods.get(&req.method_name) {
                Some(f) => Arc::clone(f),
                None => {
                    return Ok(Response::new(InvokeMethodResponse {
                        success: false,
                        result: String::new(),
                        error: format!("Method not found: {}", req.method_name),
                        metadata: None,
                    }));
                }
            }
        };

        // Parse arguments
        let args: JsonValue = match serde_json::from_str(&req.arguments) {
            Ok(v) => v,
            Err(e) => {
                return Ok(Response::new(InvokeMethodResponse {
                    success: false,
                    result: String::new(),
                    error: format!("Invalid JSON arguments: {}", e),
                    metadata: None,
                }));
            }
        };

        // Get or create context
        let context = if req.context_id.is_empty() {
            // Create temporary context for stateless calls
            ExecutionContext::new(Uuid::new_v4().to_string(), None)
        } else {
            let contexts = self.contexts.read();
            match contexts.get(&req.context_id) {
                Some(ctx) => ctx.clone(),
                None => {
                    return Ok(Response::new(InvokeMethodResponse {
                        success: false,
                        result: String::new(),
                        error: format!("Context not found: {}", req.context_id),
                        metadata: None,
                    }));
                }
            }
        };

        // Execute the function
        let result = match func(&context, args) {
            Ok(res) => res,
            Err(e) => {
                error!("Error executing {}: {}", req.method_name, e);
                return Ok(Response::new(InvokeMethodResponse {
                    success: false,
                    result: String::new(),
                    error: e,
                    metadata: None,
                }));
            }
        };

        // Calculate execution time
        let execution_time_us = start.elapsed().as_micros() as i64;

        let result_json = serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string());

        debug!(
            "Executed {} in {}μs",
            req.method_name, execution_time_us
        );

        Ok(Response::new(InvokeMethodResponse {
            success: true,
            result: result_json,
            error: String::new(),
            metadata: Some(ExecutionMetadata {
                execution_time_us,
                memory_bytes: 0, // TODO: Implement memory tracking
                runtime: "rust".to_string(),
            }),
        }))
    }

    async fn inspect_state(
        &self,
        request: Request<InspectStateRequest>,
    ) -> Result<Response<InspectStateResponse>, Status> {
        let req = request.into_inner();

        let contexts = self.contexts.read();
        match contexts.get(&req.context_id) {
            Some(context) => {
                let state = context.get_all_state();
                let state_json =
                    serde_json::to_string(&state).unwrap_or_else(|_| "{}".to_string());

                Ok(Response::new(InspectStateResponse {
                    success: true,
                    state: state_json,
                    error: String::new(),
                }))
            }
            None => Ok(Response::new(InspectStateResponse {
                success: false,
                state: String::new(),
                error: format!("Context not found: {}", req.context_id),
            })),
        }
    }

    async fn destroy_context(
        &self,
        request: Request<DestroyContextRequest>,
    ) -> Result<Response<DestroyContextResponse>, Status> {
        let req = request.into_inner();

        let removed = self.contexts.write().remove(&req.context_id).is_some();

        if removed {
            info!("Destroyed context: {}", req.context_id);
            Ok(Response::new(DestroyContextResponse {
                success: true,
                error: String::new(),
            }))
        } else {
            Ok(Response::new(DestroyContextResponse {
                success: false,
                error: format!("Context not found: {}", req.context_id),
            }))
        }
    }

    async fn list_methods(
        &self,
        request: Request<ListMethodsRequest>,
    ) -> Result<Response<ListMethodsResponse>, Status> {
        let req = request.into_inner();
        let metadata = self.metadata.read();

        let methods: Vec<MethodInfo> = metadata
            .iter()
            .filter(|(name, _)| req.prefix.is_empty() || name.starts_with(&req.prefix))
            .map(|(name, meta)| MethodInfo {
                name: name.clone(),
                description: meta.description.clone(),
                is_stateful: meta.is_stateful,
                parameter_types: meta.parameter_types.clone(),
                return_type: meta.return_type.clone(),
            })
            .collect();

        Ok(Response::new(ListMethodsResponse { methods }))
    }
}

#[derive(Parser)]
#[command(name = "transpile-test-server")]
#[command(about = "Rust gRPC server for transpilation testing")]
struct Args {
    /// Server port
    #[arg(short, long, default_value = "50052")]
    port: u16,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    let addr = format!("0.0.0.0:{}", args.port).parse()?;
    let server = TranspileTestServer::new();

    // Register example functions
    examples::register_simple_math(&server);
    info!("Registered example functions");

    info!("Rust gRPC server starting on {}", addr);
    println!("Rust gRPC server listening on port {}", args.port);

    Server::builder()
        .add_service(TranspileTestServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
