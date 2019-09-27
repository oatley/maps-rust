# maps-rust
The map generator outputs the maps in json files(option for compression). 
These map files are compatible between the [curses-map-generator](https://github.com/oatley/curses-map-generator) written in python. Gen/load with either program should work.
The rust program is at least 4x faster in map generation, though it's written with almost the same logic as the python version.
These maps are kinda random and bad right now.

# How to use:
```
Maps 1.0
Oatley
Map generator and viewer

USAGE:
    maps [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    create    create new map
    help      Prints this message or the help of the given subcommand(s)
    view      preview map in curses
```
## Create new map:
```
USAGE:
    maps create [FLAGS] --file <FILE> --size <SIZE>

FLAGS:
    -c, --compress    Compress output file with gzip
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -f, --file <FILE>    Name of file to make
    -s, --size <SIZE>    Set size of map
```
## View map in ncurses:
```
USAGE:
    maps view [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --file <FILE>    Name of file to make
```

# Examples:
## Create new map sized 100x100 and compress the file
```
git clone https://github.com/oatley/maps-rust
cd maps-rust
cargo build
target/debug/maps create --file awesome_map.map.gz --size 100 --compress
target/debug/maps view --file awesome_map.map.gz
ls -lh resources/maps # Maps stored in here
```
## Create new map sized 50x50 without compression
```
git clone https://github.com/oatley/maps-rust
cd maps-rust
cargo build
target/debug/maps create --file tiny.map --size 50
target/debug/maps view --file tiny.map
ls -lh resources/maps # Maps stored in here
```


