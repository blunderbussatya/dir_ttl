# Directory TTL Cleaner

A Rust tool that automatically cleans up directories based on Time-To-Live (TTL) values specified in their names.

## Features

- Scans specified directories for subdirectories with TTL patterns
- Supports TTL units: minutes (min), days (d), months (m), and years (y)
- Deletes expired directories based on their creation time and TTL
- Configurable via YAML file
- Logging with different verbosity levels

## Usage

1. Create a YAML configuration file:

```yaml
paths_to_watch:
  - /path/to/directory1
  - /path/to/directory2
```

2. Create directories with TTL values like:

```bash
mkdir /path/to/directory1/ttl=10d
mkdir /path/to/directory2/ttl=10m
mkdir /path/to/directory2/ttl=1y
```

3. Run the executable with the configuration file:

```bash
cargo run -- --config config.yaml
```

## License

This project is licensed under the MIT License.
