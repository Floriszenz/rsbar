# `rsbar-img`

Scan and decode bar codes from one or more image files.

## Running

```
cargo run --bin rsbar-img <images...>
```

To get a list of all options, run

```
cargo run --bin rsbar-img -- -h
```

Run the program in `release` mode to build it with performance and size optimizations:

```
cargo run --bin rsbar-img --release <images...>
```

## Differences from the Original Implementation

-   The CLI option `--verbose` is not count-based anymore and is combined with the `--quiet` option. These options are replaced with the `--verbosity` flag, which allows five options for different verbosity levels (available options: `quiet`, `minimal`, `normal` (default), `detailed`, `diagnostic`). At the moment, this does break the original behaviour as passing, e.g., the `-vminimal` flag will already print more logs than original program with no verbose flag passed (but passing `-vquiet` will reproduce the behaviour when passing `--quiet` to the original program).
-   Bugfix: Setting the config `--set binary` following the option `--xml` prints the XML and the statistics log "scanned \_ barcode symbols \[...\]", but the reverse (`--xml --set binary`) does only print the XML. As the `binary` option has no further benefit, it is removed completely (you can still set the option, but the `rsbar-img` program does not use it in any way).
-   The XML output is printed with indentation and double quotes instead of single quotes.
-   The program only prints error messages but does always terminate with exit code 0 (but this behaviour is not intended and should be fixed).
-   In the original implementation, the statistics log messages are printed to `stderr` but I decided to print it to `stdout`.
-   When scanning multiple images with the `--display` option, it opens a new window for each image as soon as the previous window was closed. In the original implementation, it is possible to abort scanning more images by pressing the `q` key. Due to changes in the code structure, this behaviour is not available anymore - you can still close the window by pressing any key or clicking the window, but it will not abort the execution and opens windows for all other images passed to the program. (This might get fixed later.)
