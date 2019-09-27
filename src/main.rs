extern crate maps;
extern crate clap;

use maps::Curses;
use maps::CursesMenu;
use maps::CursesMap;
use maps::Map;
use maps::Validation;

use clap::{Arg, App, SubCommand, AppSettings};

// What's next???
// Command line args should be eazy now that everything is shared library/abstracted!
// Test with godot once the cli works!
// Error checking?
// Test suite?



// Initialize ncurses settings, create color pairs, start main menu, start map view
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
    // Validation
    // - Is file and does not exist
    // - Is file and exists
    // - Check if file is compressed? or at least report appropriate error message
    // - maybe percentage complete (prob not)
    if let Some(matches) = matches.subcommand_matches("create") {
        let mut file_name = String::from("");
        let mut size = 0;
        let compression: bool = matches.is_present("compress");
        println!("you maeking a make");
        if matches.is_present("file") {
            let file_name = matches.value_of("file").unwrap();
            let validation = maps::Validation::new(&file_name);
            println!("checking file: {}", &file_name);
            if validation.file_exists {
                println!("error: file {} already exists", validation.file_path);
                std::process::exit(1);
            } else if ! validation.file_valid {
                println!("error: file name must use only letters and numbers");
                std::process::exit(1);
            }
        }
        if matches.is_present("size") {
            println!("size found");
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
        maps::Map::save_map(&file_name, &map, compression);
        std::process::exit(1);
    }
    // Check if file exists, compressed, valid then open in map viewer
    else if let Some(matches) = matches.subcommand_matches("view") {
        println!("you viewing a view");
        if matches.is_present("file") {
            let file_name = matches.value_of("file").unwrap();
            let v: Validation = maps::Validation::new(&file_name);
            if v.file_valid && v.file_exists {
                maps::Curses::start_curses();
                let map: maps::Map = maps::Map::load_map(&v.file_path, v.file_compressed.clone());
                let curses_map: maps::CursesMap = CursesMap::new(24, 70, map.map_wall, map.map_floor, map.map_player);
                curses_map.play_map(&map.map_game_objects);
            } else if ! v.file_exists {
                println!("error: file '{}' does not exist", v.file_path);
                std::process::exit(1);
            } else if ! v.file_valid {
                println!("error: file name must use only letters and numbers");
                std::process::exit(1);
            }
        } else {
            maps::Curses::start_curses();
            let mut v: Validation = Curses::get_map_file_name();
            let map = maps::Map::load_map(&v.file_path, v.file_compressed.clone());
            let curses_map = maps::CursesMap::new(24, 70, map.map_wall, map.map_floor, map.map_player);
            curses_map.play_map(&map.map_game_objects);
            maps::Curses::end_curses()
        }
    }
    std::process::exit(0);
}
    // New menu setup
    // let mut curses_options: Vec<String> = Vec::new();
    // curses_options.push(String::from("load map")); // 1
    // curses_options.push(String::from("quit program")); // 2
    // let menu = maps::CursesMenu::new(24, 70, curses_options);
    // let mut answer: i32 = 0;
    //
    // // Start curses menu
    // answer = menu.run();
    // if answer == '1' as i32 {
    //     menu.end();
    //
    // }
    // Curses::exit(0);
