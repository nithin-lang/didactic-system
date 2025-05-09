// Add to Cargo.toml:
// rustyline = "13"
// sysinfo = "0.30"
// git2 = "0.18"
// rand = "0.8"

use rustyline::{Editor, error::ReadlineError};
use rustyline::history::DefaultHistory;
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::path::Path;
use std::fs::{self, File};
use std::io::{Write, BufRead, BufReader};
use std::time::Instant;
use sysinfo::System;
use rand::seq::SliceRandom;
use git2::Repository;

const ALIAS_FILE: &str = "aliases.txt";
const FUN_FACTS: &[&str] = &[
    "Rust was voted the 'most loved programming language' on Stack Overflow for 7 years in a row!",
    "The Rust compiler is itself written in Rust.",
    "Rust's mascot is Ferris the crab 🦀.",
    "Rust's ownership system guarantees memory safety without garbage collection.",
    "Cargo, Rust's package manager, makes dependency management easy.",
];

fn print_ls() {
    match fs::read_dir(".") {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_type = entry.file_type();
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if let Ok(ft) = file_type {
                        if ft.is_dir() {
                            // Blue for directories
                            println!("\x1b[1;34m{}/\x1b[0m", name);
                        } else if ft.is_file() {
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let meta = entry.metadata();
                                if let Ok(meta) = meta {
                                    if meta.permissions().mode() & 0o111 != 0 {
                                        // Green for executables
                                        println!("\x1b[1;32m{}\x1b[0m", name);
                                    } else {
                                        println!("{}", name);
                                    }
                                } else {
                                    println!("{}", name);
                                }
                            }
                            #[cfg(not(unix))]
                            {
                                println!("{}", name);
                            }
                        } else {
                            println!("{}*", name);
                        }
                    } else {
                        println!("{}", name);
                    }
                }
            }
        }
        Err(e) => println!("Error reading directory: {}", e),
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

fn set_window_title(title: &str) {
    print!("\x1B]0;{}\x07", title);
    use std::io::{Write, stdout};
    let _ = stdout().flush();
}

fn print_shellinfo() {
    let mut sys = System::new_all();
    sys.refresh_all();
    println!("🦀 Rust Shell Info:");
    println!("  Rust version: {}", rustc_version_runtime::version());
    println!("  OS: {} {}", System::name().unwrap_or_default(), System::os_version().unwrap_or_default());
    println!("  Hostname: {}", System::host_name().unwrap_or_default());
    println!("  Total memory: {} MB", sys.total_memory() / 1024);
    println!("  Used memory: {} MB", sys.used_memory() / 1024);
    println!("  CPUs: {}", sys.cpus().len());

}

fn get_git_branch() -> Option<String> {
    if let Ok(repo) = Repository::discover(".") {
        if let Ok(head) = repo.head() {
            if let Some(name) = head.shorthand() {
                return Some(name.to_string());
            }
        }
    }
    None
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
        for line in BufReader::new(file).lines() {
            if let Ok(line) = line {
                if let Some((k, v)) = line.split_once('=') {
                    aliases.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        println!("Aliases loaded from {}", ALIAS_FILE);
    } else {
        println!("No alias file found.");
    }
}

fn main() {
    // Fun fact on startup
    let fact = FUN_FACTS.choose(&mut rand::thread_rng()).unwrap();
    println!("\n\x1b[1;36m🦀 Welcome to your custom Rust Shell! 🦀\x1b[0m");
    println!("\x1b[1;33m💡 Fun fact: {}\x1b[0m\n", fact);

    set_window_title("Rust Shell 🦀");

    let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
    let mut aliases = HashMap::new();
    let mut prompt_symbol = "🦀".to_string();

    // Load aliases if present
    load_aliases(&mut aliases);

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        // Git branch in prompt if present
        let git_branch = get_git_branch();
        // Example branch_str with color and no spaces:
        let branch_str = if let Some(branch) = git_branch {
            format!("\x01\x1b[1;35m\x02[{}]\x01\x1b[0m\x02", branch)
        } else {
            "".to_string()
        };

        // let prompt = format!(
        //     "\x01\x1b[1;32m\x02{} RUST-SHELL [{}]\x01\x1b[0m\x02{}> ",
        //     prompt_symbol,
        //     env::current_dir().unwrap().display(),
        //     branch_str
        // );
        let prompt = format!(
            "\x01\x1b[1;31m\x02🦀\x01\x1b[0m\x02 \x01\x1b[1;32m\x02RUST-SHELL [{}]\x01\x1b[0m\x02> ",
            prompt_symbol,
            env::current_dir().unwrap().display(),
        );
        

        
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let input = line.trim();
                let _ = rl.add_history_entry(input);

                // Expand alias (if any)
                let command_owned = if let Some(alias) = aliases.get(input) {
                    alias.clone()
                } else {
                    input.to_string()
                };
                let mut parts = command_owned.split_whitespace();
                let cmd = parts.next().unwrap_or("");
                let args: Vec<&str> = parts.collect();

                match cmd {
                    "exit" => break,
                    "help" => {
                        print_help();
                        continue;
                    }
                    "ls" => {
                        print_ls();
                        continue;
                    }
                    "cd" => {
                        if let Some(dir) = args.get(0) {
                            let path = Path::new(dir);
                            if let Err(e) = env::set_current_dir(path) {
                                println!("cd: {}", e);
                            }
                        } else {
                            println!("Usage: cd <directory>");
                        }
                        continue;
                    }
                    "setenv" => {
                        if args.len() >= 2 {
                            unsafe { env::set_var(args[0], args[1]); }
                            println!("Set {}={}", args[0], args[1]);
                        } else {
                            println!("Usage: setenv VAR VALUE");
                        }
                        continue;
                    }
                    "getenv" => {
                        if let Some(var) = args.get(0) {
                            match env::var(var) {
                                Ok(val) => println!("{}={}", var, val),
                                Err(_) => println!("{} is not set", var),
                            }
                        } else {
                            println!("Usage: getenv VAR");
                        }
                        continue;
                    }
                    "alias" => {
                        // Usage: alias name=command
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
                        continue;
                    }
                    "save-aliases" => {
                        save_aliases(&aliases);
                        continue;
                    }
                    "load-aliases" => {
                        load_aliases(&mut aliases);
                        continue;
                    }
                    "setprompt" => {
                        if let Some(sym) = args.get(0) {
                            prompt_symbol = sym.to_string();
                            println!("Prompt symbol set to '{}'", sym);
                        } else {
                            println!("Usage: setprompt <symbol>");
                        }
                        continue;
                    }
                    "shellinfo" => {
                        print_shellinfo();
                        continue;
                    }
                    "whoami-shell" => {
                        println!("🦀 You are running the custom Rust Shell!");
                        continue;
                    }
                    "" => continue, // Empty input
                    _ => {
                        // Command timing
                        let start = Instant::now();
                        let output = if cfg!(target_os = "windows") {
                            Command::new("cmd").args(&["/C", &command_owned]).output()
                        } else {
                            Command::new("sh").arg("-c").arg(&command_owned).output()
                        };
                        let elapsed = start.elapsed();

                        match output {
                            Ok(output) => {
                                print!("{}", String::from_utf8_lossy(&output.stdout));
                                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                                println!(
                                    "\x1b[1;36m[Command completed in {:.2?}]\x1b[0m",
                                    elapsed
                                );
                            }
                            Err(e) => println!("Error executing command: {}", e),
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();
    save_aliases(&aliases);
}