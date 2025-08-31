mod client;
mod server;

use client::*;
use server::*;
use std::io;
use std::thread;
use std::time::Duration;

use std::env;

fn start_server() -> io::Result<()> {
    println!("Starting nREPL server...");

    let mut server = NreplServer::new();

    match server.start_with_clj() {
        Ok(port) => {
            println!("nREPL server started successfully on port {}", port);

            println!("Server will run for 30 seconds...");
            thread::sleep(Duration::from_secs(30));

            if server.is_running() {
                println!("Server is still running. Stopping...");
                server.stop()?;
                println!("Server stopped.");
            } else {
                println!("Server has already stopped.");
            }
        }
        Err(e) => {
            eprintln!("Failed to start nREPL server: {}", e);
            eprintln!("Make sure you have Clojure CLI tools installed.");
            eprintln!("You can also try using Leiningen instead:");
            eprintln!("  let port = server.start_with_lein()?;");
        }
    }

    Ok(())
}

fn start_client(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to nREPL server...");
    let mut client = match NreplClient::connect("127.0.0.1", port) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to connect: {}", e);
            println!(
                "Make sure nREPL server is running with: lein repl :headless :host 127.0.0.1 :port 55821"
            );
            return Ok(());
        }
    };

    println!("Connected! Setting shorter timeouts for testing...");
    client.set_timeouts(Duration::from_secs(10), Duration::from_secs(5))?;

    // Test describe first
    // println!("\n=== Testing describe ===");
    // match client.describe() {
    //     Ok(desc) => {
    //         println!("Server description successful");
    //         if let Some(serde_bencode::value::Value::Dict(ops)) = desc.get("ops") {
    //             println!("Available operations: {} ops", ops.len());
    //         }
    //     }
    //     Err(e) => {
    //         println!("Describe failed: {}", e);
    //         return Ok(());
    //     }
    // }

    // Test session creation
    // println!("\n=== Testing session creation ===");
    // match client.clone_session() {
    //     Ok(session) => println!("Session created: {}", session),
    //     Err(e) => {
    //         println!("Session creation failed: {}", e);
    //         return Ok(());
    //     }
    // }

    let test_cases = vec![
        "(+ 1 2 3)",
        "(println \"Hello from Rust!\")",
        "(range 10)",
        //"(Thread/sleep 1000)", // This might timeout
        "(str \"Result: \" (+ 10 20 30))",
        "(require '[clojure.string :as str])",
        "(import (java.io File))",
        "(str \"a\" \"b\")",
    ];

    for (i, code) in test_cases.iter().enumerate() {
        println!("\n=== Test {} ===", i + 1);
        println!("Evaluating: {}", code);

        match client.eval_with_timeout(code, Duration::from_secs(5)) {
            Ok(result) => {
                println!("✓ Success!");
                if let Some(value) = &result.value {
                    println!("  Value: {}", value);
                }
                if !result.output.is_empty() {
                    println!("  Output: '{}'", result.output);
                }
                if result.has_error {
                    println!("  Error: {}", result.error);
                }
            }
            Err(NreplError::Timeout) => {
                println!("✗ Timeout - trying to interrupt...");
                match client.interrupt() {
                    Ok(_) => println!("  Interrupt sent"),
                    Err(e) => println!("  Failed to interrupt: {}", e),
                }
            }
            Err(NreplError::ConnectionClosed) => {
                println!("✗ Connection closed by server");
                break;
            }
            Err(e) => {
                println!("✗ Error: {}", e);
            }
        }

        if !client.is_connected() {
            println!("Connection lost, stopping tests");
            break;
        }
    }

    println!("\n=== Final connection check ===");
    if client.is_connected() {
        println!("Connection still alive");
    } else {
        println!("Connection closed");
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let client_or_server = &args[1].clone();

    if client_or_server == "server" {
        start_server();
    } else {
        let port: u16 = args[2]
            .clone()
            .to_string()
            .parse()
            .expect("Failed to parse port to u16");
        start_client(port);
    }
}
