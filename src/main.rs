use git2::Repository;
use rand::seq::SliceRandom;
use rustyline::history::DefaultHistory;
use rustyline::{error::ReadlineError, Editor};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use sysinfo::System;

const ALIAS_FILE: &str = "aliases.txt";
const FUN_FACTS: &[&str] = &[
    "Rust was voted the 'most loved programming language' on Stack Overflow for 7 years in a row!",
    "The Rust compiler is itself written in Rust.",
    "Rust's mascot is Ferris the crab.",
    "Rust's ownership system guarantees memory safety without garbage collection.",
    "Cargo, Rust's package manager, makes dependency management easy.",
];

fn main() {
    // Show fun fact
    show_fun_fact();

    // Set window title
    set_window_title("Rust Shell");

    // Initialize readline and alias map
    let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
    let mut aliases = HashMap::new();
    let mut prompt_symbol = String::from("*");

    // Load history and aliases
    load_aliases(&mut aliases);
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    // Main loop
    loop {
        let branch = get_git_branch();
        let prompt = format_prompt(&prompt_symbol, &branch);

        match rl.readline(&prompt) {
            Ok(line) => {
                let input = line.trim();
                let _ = rl.add_history_entry(input);

                let expanded = aliases.get(input).cloned().unwrap_or_else(|| input.to_string());
                let mut parts = expanded.split_whitespace();
                let cmd = parts.next().unwrap_or("");
                let args: Vec<&str> = parts.collect();

                if handle_internal_command(cmd, &args, &mut aliases, &mut prompt_symbol) {
                    continue;
                }

                execute_external_command(&expanded);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();
    save_aliases(&aliases);
}

fn show_fun_fact() {
    let fact = FUN_FACTS.choose(&mut rand::thread_rng()).unwrap();
    println!("\n\x1b[1;36m* Welcome to your custom Rust Shell! *\x1b[0m");
    println!("\x1b[1;33m Fun fact: {}\x1b[0m\n", fact);
}

fn format_prompt(prompt_symbol: &str, branch: &Option<String>) -> String {
    let branch_display = if let Some(branch) = branch {
        // format!("\x01\x1b[1;35m\x02[{}]\x01\x1b[0m\x02", branch)
        format!("{}", branch)
    } else {
        String::new()
    };

    format!(
        // "\x01\x1b[1;32m\x02{} RUST-SHELL [{}]{}\x01\x1b[0m\x02>",
        "\x01\x1b[1;32m\x02{} RUST-SHELL [{}]{}>",
        prompt_symbol,
        env::current_dir().unwrap().display(),
        branch_display
    )
}

fn handle_internal_command(
    cmd: &str,
    args: &[&str],
    aliases: &mut HashMap<String, String>,
    prompt_symbol: &mut String,
) -> bool {
    match cmd {
        "exit" => std::process::exit(0),
        "help" => print_help(),
        "ls" => print_ls(),
        "cd" => change_directory(args),
        "setenv" => set_env(args),
        "getenv" => get_env(args),
        "alias" => set_alias(args, aliases),
        "save-aliases" => save_aliases(aliases),
        "load-aliases" => load_aliases(aliases),
        "setprompt" => set_prompt(args, prompt_symbol),
        "shellinfo" => print_shellinfo(),
        "whoami-shell" => println!("You are running the custom Rust Shell!"),
        "" => (),
        _ => return false,
    }
    true
}

fn execute_external_command(command: &str) {
    let start = Instant::now();
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", command]).output()
    } else {
        Command::new("sh").arg("-c").arg(command).output()
    };
    let elapsed = start.elapsed();

    match output {
        Ok(output) => {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
            println!("\x1b[1;36m[Command completed in {:.2?}]\x1b[0m", elapsed);
        }
        Err(e) => println!("Error executing command: {}", e),
    }
}

fn print_ls() {
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    println!("\x1b[1;34m{}/\x1b[0m", name);
                } else if ft.is_file() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(meta) = entry.metadata() {
                            if meta.permissions().mode() & 0o111 != 0 {
                                println!("\x1b[1;32m{}\x1b[0m", name);
                            } else {
                                println!("{}", name);
                            }
                        }
                    }
                    #[cfg(not(unix))]
                    println!("{}", name);
                } else {
                    println!("{}*", name);
                }
            }
        }
    } else {
        println!("Error reading directory");
    }
}

fn change_directory(args: &[&str]) {
    if let Some(dir) = args.get(0) {
        if let Err(e) = env::set_current_dir(Path::new(dir)) {
            println!("cd: {}", e);
        }
    } else {
        println!("Usage: cd <directory>");
    }
}

fn set_env(args: &[&str]) {
    if args.len() >= 2 {
        unsafe {
            env::set_var(args[0], args[1]);
        }
        println!("Set {}={}", args[0], args[1]);
    } else {
        println!("Usage: setenv VAR VALUE");
    }
}

fn get_env(args: &[&str]) {
    if let Some(var) = args.get(0) {
        match env::var(var) {
            Ok(val) => println!("{}={}", var, val),
            Err(_) => println!("{} is not set", var),
        }
    } else {
        println!("Usage: getenv VAR");
    }
}

fn set_alias(args: &[&str], aliases: &mut HashMap<String, String>) {
    if let Some(alias_def) = args.get(0) {
        let mut split = alias_def.splitn(2, '=');
        if let (Some(name), Some(cmd)) = (split.next(), split.next()) {
            aliases.insert(name.trim().to_string(), cmd.trim().to_string());
            println!("Alias set: {}='{}'", name.trim(), cmd.trim());
        } else {
            println!("Usage: alias name=command");
        }
    } else {
        println!("Usage: alias name=command");
    }
}

fn save_aliases(aliases: &HashMap<String, String>) {
    if let Ok(mut file) = File::create(ALIAS_FILE) {
        for (k, v) in aliases {
            let _ = writeln!(file, "{}={}", k, v);
        }
        println!("Aliases saved to {}", ALIAS_FILE);
    } else {
        println!("Failed to save aliases.");
    }
}

fn load_aliases(aliases: &mut HashMap<String, String>) {
    if let Ok(file) = File::open(ALIAS_FILE) {
        for line in BufReader::new(file).lines().flatten() {
            if let Some((k, v)) = line.split_once('=') {
                aliases.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        println!("Aliases loaded from {}", ALIAS_FILE);
    } else {
        println!("No alias file found.");
    }
}

fn set_prompt(args: &[&str], prompt_symbol: &mut String) {
    if let Some(sym) = args.get(0) {
        *prompt_symbol = sym.to_string();
        println!("Prompt symbol set to '{}'", sym);
    } else {
        println!("Usage: setprompt <symbol>");
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  ls                 List files in the current directory (with colors)");
    println!("  cd <dir>           Change current directory");
    println!("  setenv VAR VAL     Set environment variable");
    println!("  getenv VAR         Get environment variable");
    println!("  alias a=b          Set alias a for command b");
    println!("  save-aliases       Save current aliases to disk");
    println!("  load-aliases       Load aliases from disk");
    println!("  setprompt <sym>    Set custom prompt symbol");
    println!("  shellinfo          Show shell and system info");
    println!("  whoami-shell       Show shell identity");
    println!("  help               Show this help");
    println!("  exit               Exit the shell");
    println!("  Any other command is run in the system shell.");
}

fn print_shellinfo() {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("  Rust Shell Info:");
    println!("  Rust version: {}", rustc_version_runtime::version());
    println!(
        "  OS: {} {}",
        System::name().unwrap_or_default(),
        System::os_version().unwrap_or_default()
    );
    println!("  Hostname: {}", System::host_name().unwrap_or_default());
    println!("  Total memory: {} MB", sys.total_memory() / 1024);
    println!("  Used memory: {} MB", sys.used_memory() / 1024);
    println!("  CPUs: {}", sys.cpus().len());
}

fn get_git_branch() -> Option<String> {
    Repository::discover(".")
        .ok()
        .and_then(|repo| repo.head().ok()?.shorthand().map(|s| s.to_string()))
}

fn set_window_title(title: &str) {
    print!("\x1B]0;{}\x07", title);
    let _ = std::io::stdout().flush();
}
