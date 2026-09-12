# One For All

**One For All (OFA)** is a lightweight CLI tool for defining reusable command macros and orchestrating shell commands through a simple TOML configuration.

Instead of maintaining multiple shell scripts, OFA lets you define commands, functions, parameters, defaults, and templates in one configuration file.

## Features

* Define reusable shell commands
* Compose commands using functions/macros
* Parameterized commands with default values
* Variable interpolation using `{{ var }}`
* Dry-run and verbose execution
* Simple TOML-based configuration
* No runtime or external service dependencies

## Example

Configuration:

```toml
[commands.build]
run = "cargo build --release"

[commands.test]
run = "cargo test"

[functions.deploy]
run = "docker compose up -d {{ service }}"
params = ["service"]
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
[functions.run]
run = "docker run {{ image }}"
params = ["image"]

[functions.logs]
run = "docker logs -f {{ container }}"
params = ["container"]
```

## Usage

```bash
ofa <command> [arguments]
```

Useful options include:

```text
--dry-run       Show commands without executing them
--verbose       Enable verbose output
--interpolate   Enable variable interpolation
```

Run:

```bash
ofa --help
```

for the complete CLI interface.

## Why OFA?

Shell scripts are great for simple automation, but larger collections of scripts can become difficult to organize and reuse.

OFA provides a small abstraction over shell commands while keeping the commands themselves simple and transparent.

Useful for managing different env, reuseable vars which are better kept in local isolation.

## License

MIT
