extern crate ncurses;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate regex;
extern crate flate2;

use ncurses::*;
use rand::Rng;
use std::time::Duration;
use std::thread;
use std::collections::HashMap;
use std::string::String;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use regex::Regex;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;

// To Do: (but probably not for a while)
// - gen/load maps from cli
// - show progress bar/percent on map generation

// Map height, width, and middle locations
const MAP_HEIGHT: i32 = 24;
const MAP_WIDTH: i32 = 70;
const MID_Y: i32 = MAP_HEIGHT / 2;
const MID_X: i32 = MAP_WIDTH / 2;
const WALL: char = '#';
const PLAYER: char = 'P';
const FLOOR: char = '.';
const COLOR_WALL: i16 = 1;
const COLOR_PLAYER: i16 = 2;
const COLOR_FLOOR: i16 = 3;

// Each tile represents a character on the map, stored in game_objects, and json files
#[derive(Serialize, Deserialize, Debug)]
struct Tile {
    y: i32,
    x: i32,
    c: char,
    neighbors: Vec<String>, // this will store a key to game_objects, for each neighbor tiles
}

// The player x and y coordinates move the view of the map, with the player at center
struct Player {
    y: i32,
    x: i32,
    c: char,
}

// Methods to move the player/map view
impl Player {
    // Draw player at middle of the window
    fn draw (&self, win: WINDOW) {
        mvwaddch(win, MID_Y, MID_X, self.c as chtype);
    }
    fn move_up (&mut self){
        self.y -= 1;
    }
    fn move_down (&mut self){
        self.y += 1;
    }
    fn move_left (&mut self){
        self.x -= 1;
    }
    fn move_right (&mut self){
        self.x += 1;
    }
}

// Determine which keys are being pressed
fn keyboard(_win: WINDOW, stats: WINDOW, ch: i32, game_objects: &HashMap<String, Tile>, player: &mut Player) {
    if ch == KEY_UP {
        // Determine the game object key of the tile above the cursor
        let key = (player.y-1).to_string() + "x" + &(player.x).to_string();
        // Check if the tile above the cursor is a wall or floor
        match game_objects.get(&key) {
            Some(tile) => {
                if tile.c == FLOOR{
                    player.move_up();
                } else { // Prevent movement through walls
                    wclear(stats);
                    mvwaddstr(stats,3,1,&key);
                }
            },
            None => {}
        }
    } else if ch == KEY_DOWN {
        // Determine the game object key of the tile below the cursor
        let key = (player.y+1).to_string() + "x" + &(player.x).to_string();
        // Check if the tile below the cursor is a wall or floor
        match game_objects.get(&key) {
            Some(tile) => {
                if tile.c == FLOOR{
                    player.move_down();
                } else { // Prevent movement through walls
                    wclear(stats);
                    mvwaddstr(stats,3,1,&key);
                }
            },
            None => {}
        }
    } else if ch == KEY_LEFT {
        // Determine the game object key of the tile left of the cursor
        let key = (player.y).to_string() + "x" + &(player.x-1).to_string();
        // Check if the tile above the cursor is a wall or floor
        match game_objects.get(&key) {
            Some(tile) => {
                if tile.c == FLOOR{
                    player.move_left();
                } else { // Prevent movement through walls
                    wclear(stats);
                    mvwaddstr(stats,3,1,&key);
                }
            },
            None => {}
        }
    } else if ch == KEY_RIGHT {
        // Determine the game object key of the tile right of the cursor
        let key = (player.y).to_string() + "x" + &(player.x+1).to_string();
        // Check if the tile above the cursor is a wall or floor
        match game_objects.get(&key) {
            Some(tile) => {
                if tile.c == FLOOR{
                    player.move_right();
                } else { // Prevent movement through walls
                    wclear(stats);
                    mvwaddstr(stats,3,1,&key);
                }
            },
            None => {}
        }
    } else if ch == KEY_RESIZE {} // This is true when the window is resized, currently not in use
}

// Gen map is used to create a variable sized map using voronoi regions (can be very slow)
fn gen_map(sizey: i32, sizex: i32) -> HashMap<String, Tile> {
    // Random numbers
    let mut rng = rand::thread_rng();
    // Generate v-regions based on size of map (# of regions matches maps.py YOU LIED HERE, HALF AS MANY!)
    let mut v_regions = Vec::new();
    let number_of_regions = rng.gen_range((sizey + sizex)/2, (sizey + sizex)*2);
    for _v in 0..number_of_regions {
        let y = rng.gen_range(0, sizey);
        let x = rng.gen_range(0, sizex);
        let tile_type = rng.gen_range(0, 2);
        if tile_type == 0 {
            v_regions.push(Tile {y: y, x: x, c: WALL, neighbors: Vec::new()});
        } else {
            v_regions.push(Tile {y: y, x: x, c: FLOOR, neighbors: Vec::new()});
        }
    }
    // Add extra region for spawn location
    v_regions.push(Tile {y: MID_Y, x: MID_X, c: FLOOR, neighbors: Vec::new()});
    // Create a new set of game_objects for modifying
    // Due to problems dereferencing and modifying game_objects, make a new structure go
    let mut go = HashMap::new();
    let mapsize = Tile {y: sizey, x: sizex, c: '$', neighbors: Vec::new()};
    let player = Tile {y: 0, x: 0, c: PLAYER, neighbors: Vec::new()};
    // Additional metadata to save in case another programs needs to know the map size
    go.insert(String::from("mapsize"), mapsize);
    go.insert(String::from("player"), player);
    // Hashmap for storing tiles and map data
    let mut game_objects = HashMap::new();
    // Generate temporary tiles
    for y in 0..sizey {
        for x in 0..sizex {
            let t = Tile {y: y, x: x, c: FLOOR, neighbors: Vec::new()};
            game_objects.insert(gen_tile_key(&t), t);
        }
    }
    // Create voronoi regions, convert tiles in game_objects to closest v-region tile type
    // This loop plus the distance calc might be slowest part of gen_map
    for k in game_objects.keys() {
        let mut closest: usize = 0;
        for i in 0..number_of_regions+1 { // Add +1 extra region for spawn location
            // Distance calc code
            let cur: usize = i as usize;
            let diff = distance(&v_regions[cur], &game_objects[k]);
            let old_diff = distance(&v_regions[closest], &game_objects[k]);
            if diff < old_diff {
                closest = cur;
            }
        }
        // Add walls on edges because why not
        let ttype: char;
        if game_objects[k].y <= 0 || game_objects[k].y >= sizey - 1 || game_objects[k].x <= 0 || game_objects[k].x >= sizex - 1 {
            ttype = WALL;
        } else {
            ttype = v_regions[closest].c;
        }
        let t = Tile {y: game_objects[k].y, x: game_objects[k].x, c: ttype, neighbors: Vec::new()};
        // Modifying game_objects tiles is a pain, so we make the changes to a mirror data structure called go
        go.insert(k.to_string(), t);
    }
    // return the modified map data structure
    go
}

// Draws the map. Only draws tiles within players view window, ignores rest of map.
fn draw_map(win: WINDOW, game_objects: &HashMap<String, Tile>, player: &Player) {
    // Use players position and map size to determine the loop size
    for ty in player.y-MID_Y..player.y+MID_Y {
        for tx in player.x-MID_X..player.x+MID_X {
            // Hashmap stores keys as string "Ypos" + "x" + "Xpos"
            let key = ty.to_string() + "x" + &tx.to_string();
            // Check if key exists to avoid crashes (this might let a corrupted map run)
            match game_objects.get(&key) {
                Some(_tile_name) => {
                    // Draw and color each map tile relative to player position
                    wattr_on(win, color_tile(game_objects[&key].c));
                    mvwaddch(win, game_objects[&key].y-player.y+MID_Y, game_objects[&key].x-player.x+MID_X, game_objects[&key].c as chtype);
                    wattr_off(win, color_tile(game_objects[&key].c));
                },
                None => {}
            }
        }
    }
    // Draw map border and player on top of the map
    box_(win, 0,0);
    wattr_on(win, COLOR_PAIR(COLOR_PLAYER));
    player.draw(win);
    wattr_off(win, COLOR_PAIR(COLOR_PLAYER));
}

// Draws a window to the right of screen with position data
fn draw_stats (win: WINDOW, player: &Player) {
    let posx = String::from("pos_x: ") + &player.x.to_string();
    let posy = String::from("pos_y: ") + &player.y.to_string();
    wclear(win);
    mvwaddstr(win, 1, 1, &posx);
    mvwaddstr(win, 2, 1, &posy);
    box_(win, 0, 0);
    wrefresh(win);
}

// Calculate distance between tiles for v-regions in gen_map
fn distance(v: &Tile, t: &Tile) -> i32 {
    let distance = (v.y - t.y).abs() + (v.x - t.x).abs();
    distance
}

// Create new tile key string, xy coordinate with separator
fn gen_tile_key(tile: &Tile) -> String {
    let ty = tile.y.to_string();
    let tx = tile.x.to_string();
    let sep = String::from("x");
    let mut s = String::new();
    s.push_str(&ty);
    s.push_str(&sep);
    s.push_str(&tx);
    s
}

// Is used to color floors, walls, and players different set colors (color pairs must be created first)
fn color_tile(c: char) -> attr_t {
    if c == WALL {
        COLOR_PAIR(COLOR_WALL)
    } else if c == FLOOR {
        COLOR_PAIR(COLOR_FLOOR)
    } else {
        COLOR_PAIR(COLOR_PLAYER)
    }
 }

// Returns a new ncurses window with specific size (main map window)
fn make_window() -> WINDOW {
    let win = newwin(MAP_HEIGHT, MAP_WIDTH, 0, 0);
    box_(win, 0, 0);
    wrefresh(win);
    win
}

// Returns ncurses window to the right of main window (little side window)
fn make_stats_windows() -> WINDOW {
    let win = newwin(5, 15, 2, MAP_WIDTH + 1);
    box_(win, 0, 0);
    wrefresh(win);
    win
}

// Opens a file for reading to decompress, deserialize, and store as hashmap
// Errors if directories don't exist
fn load_map(filename: &str) -> HashMap<String, Tile> {
    let mut s = String::new();
    let f = File::open(filename).expect("Unable to open file");
    GzDecoder::new(f).read_to_string(&mut s).unwrap();
    let game_objects: HashMap<String, Tile> = serde_json::from_str(&s).unwrap();
    game_objects
}

// Serialize hashmap into string, open a file for writing, write to file with compressed bufwriter
// Errors if directories don't exist
fn save_map (filename: &str, game_objects: &HashMap<String,Tile>) {
    let serialized = serde_json::to_string(&game_objects).unwrap();
    let f = File::create(filename).expect("Unable to create file");
    let enc = GzEncoder::new(f, Compression::default());
    let mut buf = BufWriter::new(enc);
    buf.write_all(serialized.as_bytes()).expect("Unable to write data");
}

// This exit function tries its best to fix curses terminal bugs before terminating program
fn exit(exit_code: i32) {
    clear();
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    nodelay(stdscr(), false);
    flushinp();
    echo();
    endwin();
    std::process::exit(exit_code);
}

// Menu gives the user options to generate a new map or load a saved map
fn menu () -> HashMap<String, Tile> {
    let win = make_window();
    let mut ch: i32;
    let game_objects: HashMap<String, Tile>;
    wclear(win);
    mvwaddstr(win, 1, 1, "G - generate new map");
    mvwaddstr(win, 2, 1, "L - load map");
    mvwaddstr(win, 3, 1, "Q - quit program");
    box_(win, 0, 0);
    wrefresh(win);
    loop {
        ch = wgetch(win);
        if ch == 'g' as i32 || ch == 'G' as i32 {
            // Get filename and size from user, generate map, write to file (possible race condition, don't care)
            let path: String = get_map_filename(true);
            let size: i32 = get_map_size();
            game_objects = gen_map(size, size);
            save_map(&path, &game_objects);
            break;
        } else if ch == 'l' as i32 || ch == 'L' as i32 {
            // Get filename from user, try loading the map (should probably check for errors, but won't)
            let path: String = get_map_filename(false);
            game_objects = load_map(&path);
            break;
        } else if ch == 'q' as i32 || ch == 'Q' as i32 {
            // Exit program
            exit(0);
        }
    }
    endwin();
    game_objects
}

// Create new window to get filename string from user, validate string, return filename string
// Used for both map generation and map loading: uses check_exists to switch between each use case
fn get_map_filename(check_exists: bool) -> String {
    let win = make_window();
    let mut filename  = String::new();
    let mut path;
    let q = String::from("Enter a filename: ");
    let qlen = q.len() as i32;
    let re = Regex::new(r"^[\w\d]+$").unwrap();
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    echo();
    wclear(win);
    loop {
        // Clear line
        wmove(win, 1, 1);
        wclrtoeol(win);
        // Ask question, get string
        mvwaddstr(win, 1, 1, &q);
        box_(win, 0, 0);
        wrefresh(win);
        mvwgetstr(win, 1, qlen, &mut filename);
        path = String::from("./resources/maps/") + &filename + ".map";
        // Used for map generation, get filename, file must not already exist
        if check_exists {
            if Path::new(&path).exists() {
                // Clear line
                wmove(win, 2, 1);
                wclrtoeol(win);
                // Write error message
                let s = String::from("error: file ") + &path + " already exists";
                mvwaddstr(win, 2, 1, &s);
                // Reset filename input
                path.clear();
                filename.clear();
                continue;
            }
        // Used for map loading, get filename, file must exist already
        } else if ! check_exists {
            if ! Path::new(&path).exists() {
                // Clear line
                wmove(win, 2, 1);
                wclrtoeol(win);
                // Write error message
                let s = String::from("error: file ") + &path + " does not exist";
                mvwaddstr(win, 2, 1, &s);
                // Reset filename input
                path.clear();
                filename.clear();
                continue;
            }
        }
        // check if filename contains non-letter or non-numbers (error out) no special characters please
        if ! re.is_match(&filename) {
            // Clear line
            wmove(win, 2, 1);
            wclrtoeol(win);
            // Write error message
            let s = String::from("error: filename ") + &filename + " must contain only letters and numbers";
            mvwaddstr(win, 2, 1, &s);
            // Reset filename input
            path.clear();
            filename.clear();
            continue;
        } else {
            break;
        }
    }
    // Destroy window, return valid path
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    noecho();
    endwin();
    path
}

// Menu to ask user for size input, check if valid number, return number
fn get_map_size() -> i32 {
    let win = make_window();
    let mut size  = String::new();
    let mut num_size: i32;
    //Size of map must be 50 or greater.
    let q = String::from("Size: ");
    let qlen = q.len() as i32;
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    echo();
    wclear(win);
    loop {
        // Clear top two lines
        wmove(win, 1, 1);
        wclrtoeol(win);
        wmove(win, 2, 1);
        wclrtoeol(win);
        // Ask question, get string
        mvwaddstr(win, 1, 1, "Size of map must be 50 or greater.");
        mvwaddstr(win, 2, 1, &q);
        box_(win, 0, 0);
        wrefresh(win);
        mvwgetstr(win, 1, qlen, &mut size);
        // Check if valid integer >= 50
        match size.trim().parse::<i32>() {
            Ok(number) => {
                num_size = number;
                if num_size < 50 {
                    wmove(win, 2, 1);
                    wclrtoeol(win);
                    let s = String::from("error: ") + &size + " must be >= 50";
                    mvwaddstr(win, 2, 1, &s);
                    size.clear();
                } else {
                    break;
                }
            },
            Err(_error) => {
                wmove(win, 2, 1);
                wclrtoeol(win);
                let s = String::from("error: ") + &size + " not integer";
                mvwaddstr(win, 2, 1, &s);
                size.clear();
            },
        }
    }
    // Destroy window, return valid int
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    endwin();
    num_size
}

// Get player input, draw map view, main loop
fn play_map(game_objects: HashMap<String, Tile>) {
    let win = make_window();
    let stats = make_stats_windows();
    nodelay(win, true);
    // Create player and add offset position
    let mut player = Player {y: MID_Y, x: MID_X, c: PLAYER};
    draw_map(win, &game_objects, &player);
    wrefresh(win);
    let mut ch = 0;
    while ch != 'q' as i32 {
        ch = getch(); // Reduce flickering by wgetch in separate window?
        flushinp();
        // Force map to always be same size, no matter if window resized
        wresize(win, MAP_HEIGHT, MAP_WIDTH);
        keyboard(win, stats, ch, &game_objects, &mut player);
        draw_map(win, &game_objects, &player);
        draw_stats(stats, &player);
        wnoutrefresh(win);
        thread::sleep(Duration::from_millis(30)); // reduces screen flicker a little, slows player input
        wclear(win);
        doupdate();
    }
    exit(0);
}

// Initialize ncurses settings, create color pairs, start main menu, start map view
fn main() {
    initscr();
    raw();
    keypad(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    noecho();
    // Enable curses colors
    start_color();
    use_default_colors();
    // Create custom color pairs for: player, floors, and walls
    init_pair(COLOR_WALL, 57, 234);
    init_pair(COLOR_PLAYER, 35, 0);
    init_pair(COLOR_FLOOR, 60, 0);
    // BUG if refresh is not run at least once, no windows refresh works (uhh?)
    refresh();
    // Start the main menu, which either generates/loads a map
    let game_objects: HashMap<String, Tile> = menu();
    // Start the main loop for viewing map
    play_map(game_objects);
}
