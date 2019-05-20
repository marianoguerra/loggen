# loggen: Generate logs from a directory tree of sample logs

loggen is a CLI tool that takes an input base directory, finds all files in it
and reads each line of each file and writes it to the same path in an output
base directory.

When it reaches the end of the file it starts again.

Stop with `Ctrl+C`

## Options

`loggen --help`

```
loggen
Generates log lines from a folder structure of samples

USAGE:
    loggen [OPTIONS] --in-base-dir <FILE> --out-base-dir <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --in-base-dir <FILE>     Input base directory
    -t, --interval <MS>          Time in milliseconds between line generation [default: 250]
    -o, --out-base-dir <FILE>    Output base directory
    -p, --parallelism <COUNT>    Number of parallel generators [default: 2]
```

## Example usage

There is a sample input directory in data, replace `in-dir-path` for `data` to
try it with them.

You can watch the results written in `out-dir-path` with `tail -F output-dir-path/*/*/*.log`

### Simple

Sleep 250ms between reads, use as many threads as available cores

```
loggen -i in-dir-path -o out-dir-path
```

### Two threads, sleep 1ms between reads

Sleep 1ms between reads, use 2 threads

```
loggen -i in-dir-path -o out-dir-path -p 2 -t 1
```

## Build

You need rust, check https://rustup.rs/ for installation instructions.

### Run in development

```
cargo run -- -i in-dir-path -o out-dir-path
```

### Release build

Result is in target/release/loggen


```
cargo build --release
```

## License

MIT
