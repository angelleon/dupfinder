# CLI duplicate file finder

## Build from source
1. Clone the repo
```bash
    git clone https://github.com/angelleon/dupfinder.git
```

2. Build source files using cargo
```bash
    cd dupfinder
    cargo build --release
```

3. Run the binary giving a list of files or directories to search recursively
```bash
    cd target/relase
    ./dupfinder FILE_OR_DIRECTORY
```

## Usage 
```txt
    Usage: dupfinder [OPTIONS] [PATH]...

    Arguments:
    [PATH]...

    Options:
    -s, --min-size <MIN_SIZE>  [default: 0]
    -S, --max-size <MAX_SIZE>  [default: 18446744073709551615]
    -v, --verbose
    -h, --help                 Print help information
    -V, --version              Print version information

```