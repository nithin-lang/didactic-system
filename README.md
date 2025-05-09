# 🦀 Rust Shell

A minimal, customizable command-line shell written in Rust — with features like aliases, environment management, history, prompt customization, and fun facts about Rust on startup. 🧠✨

---

## 🚀 Features

- 💡 Random Rust fun fact on launch
- 🧠 Alias support (with save/load functionality)
- 📂 `cd`, `ls` and environment variable handling (`setenv`, `getenv`)
- 🧾 Shell history (auto-saved)
- 🧰 Runs system commands cross-platform (Windows/Linux/macOS)
- 🧠 Git branch detection in prompt (if inside a repo)
- ✨ Colored directory listing (on Unix)
- 🔍 Built-in commands like `help`, `whoami-shell`, `shellinfo`
- 🎨 Custom prompt symbol with `setprompt`

---

## 📦 Installation

### 1. Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (with `cargo`)
- Git (optional, for branch detection)
- Build tools (for your OS)

### 2. Clone and Build

```bash
git clone https://github.com/yourusername/rust-shell.git
cd rust-shell
cargo build --release
```

### 3. Run

```bash
cargo run
```

Or run the compiled executable:

```bash
./target/release/rust-shell
```

---

## ⚙️ Usage

You can run commands just like in a regular shell:

```bash
> ls
> cd my_folder
> setenv NAME Rustacean
> getenv NAME
> alias gs=git status
> gs
```

### 📜 Built-in Commands

| Command            | Description                                  |
|--------------------|----------------------------------------------|
| `ls`               | Lists directory contents (color-coded)       |
| `cd <dir>`         | Changes directory                            |
| `setenv VAR VAL`   | Sets an environment variable                 |
| `getenv VAR`       | Gets an environment variable                 |
| `alias a=b`        | Creates an alias                             |
| `save-aliases`     | Saves aliases to `aliases.txt`               |
| `load-aliases`     | Loads aliases from `aliases.txt`             |
| `setprompt <sym>`  | Sets custom prompt symbol                    |
| `shellinfo`        | Displays system and shell info               |
| `whoami-shell`     | Prints shell identity                        |
| `help`             | Lists available commands                     |
| `exit`             | Exits the shell                              |

---

## 📁 Files

- `src/main.rs` - Main source code
- `aliases.txt` - Saved aliases
- `history.txt` - Shell history

---

## 🤓 Fun Fact Sample Output

```bash
* Welcome to your custom Rust Shell! *
 Fun fact: Rust was voted the 'most loved programming language' on Stack Overflow for 7 years in a row!
```

---

## 🖥 Screenshot

> *(Optional: You can add a screenshot of the terminal interface here)*

---

## 📜 License

This project is licensed under the MIT License.

---

## 🤝 Contributions

Pull requests are welcome! Feel free to open issues or feature requests.

---

## 💡 Credits

Built with ❤️ and `rustyline`, `git2`, `sysinfo`, and pure Rust magic.