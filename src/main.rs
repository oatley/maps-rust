extern crate maps;
extern crate clap;

use clap::{Arg, App, SubCommand, AppSettings};

// What's next:
// Command line args (Done kinda)
// Support stdout/stderr, there is not very much output, but currently might not work
// Add some loading info or a timer?
// Test godot GDNative shared library

// Initialize command line argument parser and run the program
fn main() {
    // Command line options
    let matches = App::new("Maps")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .version("1.0")
                        .author("Oatley")
                        .about("Map generator and viewer")
                        .subcommand(SubCommand::with_name("create")
                                    .about("create new map")
                                    .arg(Arg::with_name("file")
                                        .short("f")
                                        .long("file")
                                        .value_name("FILE")
                                        .help("Name of file to make")
                                        .takes_value(true)
                                        .required(true))
                                    .arg(Arg::with_name("size")
                                        .short("s")
                                        .long("size")
                                        .value_name("SIZE")
                                        .help("Set size of map")
                                        .takes_value(true)
                                        .required(true))
                                    .arg(Arg::with_name("compress")
                                        .short("c")
                                        .long("compress")
                                        .help("Compress output file with gzip")))
                        .subcommand(SubCommand::with_name("view")
                                    .about("preview map in curses")
                                    .arg(Arg::with_name("file")
                                        .short("f")
                                        .long("file")
                                        .value_name("FILE")
                                        .help("Name of file to make")
                                        .takes_value(true)))
                        .get_matches();
    // Create is used to create a new map and save it to a file
    if let Some(matches) = matches.subcommand_matches("create") {
        let mut file_path = String::new();
        let mut size = 0;
        let compression: bool = matches.is_present("compress");
        if matches.is_present("file") {
            let file = matches.value_of("file").unwrap();
            let validation = maps::Validation::new(&file);
            if validation.file_exists {
                println!("error: file {} already exists", &validation.file_path);
                std::process::exit(1);
            } else if ! validation.file_valid {
                println!("error: file name must use only letters and numbers");
                std::process::exit(1);
            }
            file_path = validation.file_path.clone();
        }
        if matches.is_present("size") {
            match matches.value_of("size").unwrap().trim().parse::<i32>() {
                Ok(number) => {
                    size = number;
                    if size < 49 {
                        println!("error: size must be integer 50 or larger");
                        std::process::exit(1);
                    }
                },
                Err(_error) => {
                    println!("error: size must be integer 50 or larger");
                    std::process::exit(1);
                },
            }
        }
        // gen map with data recieved
        let map = maps::Map::new(size, size, '#', '.', 'p');
        maps::Map::save_map(&file_path, &map, compression);
        std::process::exit(0);
    }
    // View is used to view a previously generated map in a ncurses window viewer
    else if let Some(matches) = matches.subcommand_matches("view") {
        if matches.is_present("file") {
            let file_name = matches.value_of("file").unwrap();
            let validation = maps::Validation::new(&file_name);
            if validation.file_valid && validation.file_exists {
                maps::Curses::start_curses();
                let map: maps::Map = maps::Map::load_map(&validation.file_path, validation.file_compressed.clone());
                let curses_map: maps::CursesMap = maps::CursesMap::new(24, 70, map.map_wall, map.map_floor, map.map_player);
                curses_map.play_map(&map.map_game_objects);
            } else if ! validation.file_exists {
                println!("error: file '{}' does not exist", &validation.file_path);
                std::process::exit(1);
            } else if ! validation.file_valid {
                println!("error: file name must use only letters and numbers");
                std::process::exit(1);
            }
        } else {
            maps::Curses::start_curses();
            let validation: maps::Validation = maps::Curses::get_map_file_name();
            let map = maps::Map::load_map(&validation.file_path, validation.file_compressed.clone());
            let curses_map = maps::CursesMap::new(24, 70, map.map_wall, map.map_floor, map.map_player);
            curses_map.play_map(&map.map_game_objects);
            maps::Curses::end_curses()
        }
    }
    std::process::exit(0);
}
