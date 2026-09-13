# One For All

**One For All (ofa)** is a lightweight CLI tool for defining reusable command macros and orchestrating shell commands through a simple TOML configuration.

Instead of maintaining multiple shell scripts, OFA lets you define commands, functions, parameters, defaults, and templates in one configuration file.

## Features

* Define reusable shell commands
* Compose commands using functions/macros
* Parameterized commands with default values
* Variable interpolation using `{{ var }}`
* Dry-run and verbose execution
* Simple TOML-based configuration
* No runtime or external service dependencies

## Installation

### Build from source

OFA requires the Rust toolchain to build from source.

Clone the repository and build the release binary:

```bash
git clone <repository-url>
cd ofa

cargo install --path .
```

Verify:

```bash
ofa --version
```

### Configuration

After installation, create the OFA configuration directory:

```bash
mkdir -p ~/.config/ofa
```

Create the `global` configuration file:

```bash
touch ~/.config/ofa/global.toml
```

In a similar fashion create the `local` configuration file: `.ofa.toml` and optional `profile` at `~/.config/ofa/profiles/<profile-name>.toml`.

OFA reads the commands and functions in the following order:

`local -> profile -> global` 

Configs read earlier cannot be overwritten so `local` can override `profile` which can again override `global`.

```text
~/.config/ofa/global.toml
```

## Example

Configuration:

```toml
[commands.build]
run = "cargo build --release"

[commands.test]
run = "cargo test"

[commands.deploy]
params = ["service"]
run = "docker compose up -d {{ service }}"
```

Run commands with:

```bash
ofa build
ofa test
ofa deploy api
```

## Configuration

OFA uses a TOML configuration file located at:

```text
~/.ofa/global.toml
```

Commands and functions can be defined with parameters and optional defaults:

```toml
[commands.run]
params = ["image"]
run = "docker run {{ image }}"

[commands.logs]
params = ["container"]
run = "docker logs -f {{ container }}"
```

## Usage

```bash
ofa <command> [arguments]
```

Useful options include:

```text
--dry-run       Show commands without executing them
--verbose       Enable verbose output, print the command before execution
--interpolate   Enable variable interpolation; for dry-run, controls env var interpolation
```

Run:

```bash
ofa --help
```

for the complete CLI interface.

## Why OFA?

Shell scripts are great for simple automation, but larger collections of scripts can become difficult to organize and reuse.

OFA provides a small abstraction over shell commands while keeping the commands themselves simple and transparent.

Useful for managing different environments and reusable variables while keeping configuration isolated to the local machine.

## License

MIT
