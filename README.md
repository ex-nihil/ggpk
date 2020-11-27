# Path of Exile GGPK reader

A CLI and library for reading GGPK files for the game [Path of Exile](pathofexile.com/) created by [Grinding Gear Games](https://www.grindinggear.com/).


## Building

Instructions for setting up for rust development:
https://www.rust-lang.org/learn/get-started

After your rustc and cargo is installed simply run the following:
```sh
$ cargo build --release
```

## Usage

#### list all files
```sh
$ ggpk --path "C:/games/Path of Exile/"
```

#### filtered listing
```sh
$ ggpk --path "C:/games/Path of Exile/" --query .+/Data/.+.bundle.bin
```
Regex format used: https://docs.rs/regex/1.4.2/regex/

#### write files to disk
```sh
$ ggpk --path "C:/games/Path of Exile/" --output "C:/GGPK_Dump/"
```
Can be combined with the query parameter to write only some files.

#### write to stdout
Note: Won't work from Windows as terminal only allows valid utf-8 symbols to be written.

Requires specifying a query resulting in a single file matching.
Useful for pipeing a single file to another process.
```sh
$ ggpk --path "/games/Path of Exile" -q .+/_.index.bin --binary
```