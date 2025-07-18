use std::pin::Pin;
use std::future::Future;
use serde_json::Value;

// Re-export inventory for macro use
pub use inventory;

/// Errors that can occur during function execution
#[derive(Debug, thiserror::Error)]
pub enum RaycastError {
    #[error("Missing argument for function '{function}', parameter '{parameter}' at position {position}")]
    MissingArgument {
        function: String,
        parameter: String,
        position: usize,
    },

    #[error("Argument count mismatch for function '{function}': expected {expected}, got {actual}")]
    ArgumentCountMismatch {
        function: String,
        expected: usize,
        actual: usize,
    },

    #[error("Failed to decode parameter '{parameter}' at position {position} for function '{function}': {error}")]
    DecodingError {
        function: String,
        parameter: String,
        position: usize,
        error: String,
    },

    #[error("Function '{function}' not found")]
    FunctionNotFound {
        function: String,
    },

    #[error("Function execution failed: {error}")]
    ExecutionError {
        error: String,
    },

    #[error("JSON parsing error: {error}")]
    JsonError {
        error: String,
    },
}

/// A registered Raycast function
pub struct RaycastFunction {
    pub name: &'static str,
    pub execute: fn(String, Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, RaycastError>> + Send + 'static>>,
}

inventory::collect!(RaycastFunction);

/// Main executor for Raycast functions
pub struct RaycastExecutor;

impl RaycastExecutor {
    /// Execute a function by name with the given arguments
    pub async fn execute(function_name: &str, args: Vec<Value>) -> Result<Value, RaycastError> {
        // Find the function in the registry
        let function = inventory::iter::<RaycastFunction>()
            .find(|f| f.name == function_name)
            .ok_or_else(|| RaycastError::FunctionNotFound {
                function: function_name.to_string(),
            })?;

        // Execute the function
        (function.execute)(function_name.to_string(), args).await
    }

    /// Run the main CLI loop
    pub async fn run_cli() -> Result<(), RaycastError> {
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 2 {
            eprintln!("Usage: {} <function_name> [args...]", args[0]);
            std::process::exit(1);
        }

        let function_name = &args[1];

        // Read JSON arguments from stdin
        let mut input = String::new();
        use std::io::Read;
        std::io::stdin().read_to_string(&mut input).map_err(|e| {
            RaycastError::JsonError {
                error: format!("Failed to read from stdin: {}", e),
            }
        })?;

        // Parse JSON arguments
        let json_args: Vec<Value> = if input.trim().is_empty() {
            vec![]
        } else {
            serde_json::from_str(&input).map_err(|e| RaycastError::JsonError {
                error: format!("Failed to parse JSON arguments: {}", e),
            })?
        };

        // Execute the function
        match Self::execute(function_name, json_args).await {
            Ok(result) => {
                println!("{}", serde_json::to_string(&result).unwrap());
                Ok(())
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// Helper functions for converting results to JSON values
pub fn serialize_to_json<T: serde::Serialize>(value: T) -> Result<Value, RaycastError> {
    serde_json::to_value(value).map_err(|e| RaycastError::ExecutionError {
        error: format!("Failed to serialize result: {}", e),
    })
}

pub fn serialize_result_to_json<T, E>(result: Result<T, E>) -> Result<Value, RaycastError>
where
    T: serde::Serialize,
    E: std::fmt::Display,
{
    match result {
        Ok(value) => serialize_to_json(value),
        Err(e) => Err(RaycastError::ExecutionError {
            error: e.to_string(),
        }),
    }
}
