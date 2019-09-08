# maps-rust
The map generator outputs the maps in compressed(gzip) json files. 
These map files are compatible between the curses-map-generator written in python. Gen/load with either program should work.
The rust program is at least 4x faster in map generation, though it's written with almost the same logic as the python version.

# How to use:
clone the git
make sure you have rust
cd into directory
cargo run


