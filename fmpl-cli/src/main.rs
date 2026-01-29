//! FMPL Command-Line REPL

use fmpl_core::debug;
use fmpl_core::stream::StreamEvent;
use fmpl_core::{Value, Vm, eval, is_complete};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::sync::Mutex;

/// Block and wait for an async stream to complete.
/// Returns the final result value or an error.
pub fn wait_for_async(value: Value) -> Result<Value, String> {
    match value {
        Value::AsyncStream(handle) => {
            let mut handle = handle.lock().map_err(|e| format!("Lock error: {}", e))?;

            // Collect all events from the stream
            let mut final_value = Value::Null;

            loop {
                match handle.recv_blocking() {
                    Some(StreamEvent::Data(v)) => {
                        // Intermediate data - keep last value
                        final_value = v;
                    }
                    Some(StreamEvent::Ok(v)) => {
                        // Terminal success - return result
                        return Ok(v);
                    }
                    Some(StreamEvent::Err(e)) => {
                        // Terminal error - return error
                        return Err(format!("Async error: {}", e));
                    }
                    Some(StreamEvent::Done) => {
                        // Stream completed without value - return final data or null
                        if final_value != Value::Null {
                            return Ok(final_value);
                        }
                        return Ok(Value::Null);
                    }
                    None => {
                        // Channel closed without Ok/Err/Done
                        if final_value != Value::Null {
                            return Ok(final_value);
                        }
                        return Err("Async stream completed without result".to_string());
                    }
                }
            }
        }
        _ => Ok(value),
    }
}

fn main() -> rustyline::Result<()> {
    // Create a tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let handle = runtime.handle();

    println!("FMPL v0.1.0");
    println!("Type .help for commands, .quit to exit");
    println!();

    let mut rl = DefaultEditor::new()?;

    // Load history if it exists
    let history_path = dirs::home_dir()
        .map(|h| h.join(".fmpl_history"))
        .unwrap_or_default();
    let _ = rl.load_history(&history_path);

    let mut vm = Vm::with_runtime(handle.clone());

    let mut input_buffer = String::new();
    let mut continuation = false;

    // Store last input for debugging
    let last_input = Mutex::new(String::new());

    loop {
        let prompt = if continuation { "....> " } else { "fmpl> " };
        match rl.readline(prompt) {
            Ok(line) => {
                let trimmed = line.trim();

                // Handle empty line
                if trimmed.is_empty() {
                    if continuation {
                        // In continuation mode, empty line submits what we have
                        // (or cancels if buffer is empty)
                        if input_buffer.trim().is_empty() {
                            input_buffer.clear();
                            continuation = false;
                        }
                        // Otherwise just add a newline to the buffer
                        input_buffer.push('\n');
                    }
                    continue;
                }

                // Add to input buffer
                if continuation {
                    input_buffer.push('\n');
                }
                input_buffer.push_str(&line);

                // Check for REPL commands (only on first line)
                if !continuation && trimmed.starts_with('.') {
                    let _ = rl.add_history_entry(&input_buffer);
                    match handle_command(&mut vm, trimmed, &last_input) {
                        CommandResult::Continue => {}
                        CommandResult::Quit => break,
                    }
                    input_buffer.clear();
                    continue;
                }

                // Check if input is complete
                match is_complete(&input_buffer) {
                    Ok(true) => {
                        // Input is complete, evaluate it
                        let _ = rl.add_history_entry(&input_buffer);

                        // Store for debugging
                        if let Ok(mut last) = last_input.lock() {
                            *last = input_buffer.clone();
                        }

                        match eval(&mut vm, &input_buffer) {
                            Ok(value) => {
                                // Check if value is an async stream that needs blocking wait
                                let display_value =
                                    if matches!(value, fmpl_core::Value::AsyncStream(_)) {
                                        // Block and wait for async result
                                        match wait_for_async(value) {
                                            Ok(result) => result,
                                            Err(e) => {
                                                eprintln!("Error waiting for async: {}", e);
                                                input_buffer.clear();
                                                continuation = false;
                                                continue;
                                            }
                                        }
                                    } else {
                                        value
                                    };

                                println!("=> {}", display_value);
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }

                        input_buffer.clear();
                        continuation = false;
                    }
                    Ok(false) => {
                        // Input is incomplete, continue reading
                        continuation = true;
                    }
                    Err(e) => {
                        // Syntax error that can't be fixed
                        eprintln!("Error: {}", e);
                        input_buffer.clear();
                        continuation = false;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                input_buffer.clear();
                continuation = false;
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Bye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&history_path);

    Ok(())
}

enum CommandResult {
    Continue,
    Quit,
}

fn handle_command(vm: &mut Vm, line: &str, last_input: &Mutex<String>) -> CommandResult {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let cmd = parts[0];
    let _arg = parts.get(1).copied().unwrap_or("");

    match cmd {
        ".quit" | ".q" | ".exit" => CommandResult::Quit,

        ".help" | ".h" | ".?" => {
            println!("FMPL REPL Commands:");
            println!("  .help, .h, .?     Show this help");
            println!("  .quit, .q, .exit  Exit the REPL");
            println!("  .clear            Clear the screen");
            println!("  .reset            Reset the VM state");
            println!("  .objects          List all named objects");
            println!("  .debug            Show debug info for last input");
            println!();
            println!("FMPL Quick Reference:");
            println!("  let (x = 42) x + 1       Let binding");
            println!("  lambda (x, y) x + y      Lambda expression");
            println!("  \\x x + 1                 Short lambda");
            println!("  if cond then a else b    Conditional");
            println!("  [1, 2, 3]                List literal");
            println!("  %{{foo: 1, bar: 2}}        Map literal");
            println!("  x |> f |> g              Pipe operator");
            println!("  obj.method(args)         Method call");
            println!("  obj.property             Property access");
            CommandResult::Continue
        }

        ".clear" => {
            print!("\x1B[2J\x1B[1;1H");
            CommandResult::Continue
        }

        ".reset" => {
            *vm = Vm::new();
            println!("VM state reset.");
            CommandResult::Continue
        }

        ".objects" => {
            println!("Named objects:");
            let mut count = 0;
            for (name, _id) in vm.objects.lock().unwrap().named_objects() {
                println!("  {}", name);
                count += 1;
            }
            if count == 0 {
                println!("  (none)");
            }
            CommandResult::Continue
        }

        ".debug" => {
            // Show debug info for last input
            let last = match last_input.lock() {
                Ok(l) => l.clone(),
                Err(_) => {
                    eprintln!("Error accessing last input");
                    return CommandResult::Continue;
                }
            };

            if last.is_empty() {
                println!("No previous input to debug.");
                return CommandResult::Continue;
            }

            println!("=== Debug Info for Last Input ===");
            println!();
            println!("Source ({} bytes):", last.len());
            if last.len() > 200 {
                println!("{}", debug::format_with_lines(&last[..200]));
                println!("... ({} more bytes)", last.len() - 200);
            } else {
                println!("{}", debug::format_with_lines(&last));
            }
            println!();

            // Show tokenization
            println!("=== Tokenization ===");
            let tokens = debug::debug_tokenize(&last);
            for token in &tokens {
                println!("{}", token);
            }
            println!();

            // Try to parse and show result
            println!("=== Parse Result ===");
            let parse_result = debug::debug_parse(&last, false);
            if parse_result.success {
                println!("Parse successful");
            } else {
                println!("Parse failed");
                if let Some(error) = parse_result.error_message {
                    println!("Error: {}", error);
                }
            }
            CommandResult::Continue
        }

        _ => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("Type .help for available commands.");
            CommandResult::Continue
        }
    }
}
