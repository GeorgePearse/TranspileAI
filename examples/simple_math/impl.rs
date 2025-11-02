/*!
Example implementation: Simple math functions (Rust)

This module demonstrates how to write Rust functions that can be tested
via the transpilation testing infrastructure.

This should be integrated into the Rust server for testing.
*/

use serde_json::{json, Value as JsonValue};

pub fn register_functions(server: &crate::TranspileTestServer) {
    // Add
    server.register_function(
        "add",
        |_ctx, args| {
            let a = args["a"].as_i64().ok_or("Missing or invalid 'a'")?;
            let b = args["b"].as_i64().ok_or("Missing or invalid 'b'")?;
            Ok(json!(a + b))
        },
        "Add two numbers",
        false,
        vec!["int".to_string(), "int".to_string()],
        "int",
    );

    // Multiply
    server.register_function(
        "multiply",
        |_ctx, args| {
            let a = args["a"].as_i64().ok_or("Missing or invalid 'a'")?;
            let b = args["b"].as_i64().ok_or("Missing or invalid 'b'")?;
            Ok(json!(a * b))
        },
        "Multiply two numbers",
        false,
        vec!["int".to_string(), "int".to_string()],
        "int",
    );

    // Fibonacci
    server.register_function(
        "fibonacci",
        |_ctx, args| {
            let n = args["n"].as_i64().ok_or("Missing or invalid 'n'")?;

            if n <= 1 {
                return Ok(json!(n));
            }

            let mut a = 0i64;
            let mut b = 1i64;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }

            Ok(json!(b))
        },
        "Calculate the nth Fibonacci number",
        false,
        vec!["int".to_string()],
        "int",
    );

    // Counter increment (stateful)
    server.register_function(
        "counter_increment",
        |ctx, _args| {
            let current = ctx
                .get_state("counter")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let new_value = current + 1;
            ctx.set_state("counter".to_string(), json!(new_value));

            Ok(json!(new_value))
        },
        "Increment a counter (stateful)",
        true,
        vec![],
        "int",
    );

    // Counter get (stateful)
    server.register_function(
        "counter_get",
        |ctx, _args| {
            let current = ctx
                .get_state("counter")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            Ok(json!(current))
        },
        "Get current counter value (stateful)",
        true,
        vec![],
        "int",
    );

    // Factorial
    server.register_function(
        "factorial",
        |_ctx, args| {
            let n = args["n"].as_i64().ok_or("Missing or invalid 'n'")?;

            fn factorial_recursive(n: i64) -> i64 {
                if n <= 1 {
                    1
                } else {
                    n * factorial_recursive(n - 1)
                }
            }

            Ok(json!(factorial_recursive(n)))
        },
        "Calculate factorial of a number",
        false,
        vec!["int".to_string()],
        "int",
    );

    // Is prime
    server.register_function(
        "is_prime",
        |_ctx, args| {
            let n = args["n"].as_i64().ok_or("Missing or invalid 'n'")?;

            if n < 2 {
                return Ok(json!(false));
            }
            if n == 2 {
                return Ok(json!(true));
            }
            if n % 2 == 0 {
                return Ok(json!(false));
            }

            let limit = (n as f64).sqrt() as i64;
            for i in (3..=limit).step_by(2) {
                if n % i == 0 {
                    return Ok(json!(false));
                }
            }

            Ok(json!(true))
        },
        "Check if a number is prime",
        false,
        vec!["int".to_string()],
        "bool",
    );
}
