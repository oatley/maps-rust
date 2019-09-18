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

// The player x and y coordinates move the view of the map, with the player at center
pub struct Player {
    y: i32,
    x: i32,
    c: char,
}

// Each tile represents a character on the map, stored in game_objects, and json files
#[derive(Serialize, Deserialize, Debug)]
pub struct Tile {
    pub y: i32,
    pub x: i32,
    pub c: char,
    pub neighbors: Vec<String> // this will store a key to game_objects, for each neighbor tiles
}

// Used to draw windows on screen
pub struct Curses {
    pub height: i32,
    pub width: i32,
    pub curse_wall: char,
    pub curse_floor: char,
    pub curse_player: char,
    pub curse_color_wall: i16,
    pub curse_color_floor: i16,
    pub curse_color_player: i16
}

pub struct Map {
    pub map_wall: char,
    pub map_floor: char,
    pub map_player: char,
    pub map_game_objects: HashMap<String, Tile>
}

// Methods to move the player/map view
impl Player {
    pub fn move_up (&mut self){
        self.y -= 1;
    }
    pub fn move_down (&mut self){
        self.y += 1;
    }
    pub fn move_left (&mut self){
        self.x -= 1;
    }
    pub fn move_right (&mut self){
        self.x += 1;
    }
    // Determine which keys are being pressed
    pub fn get_keyboard_input(_win: WINDOW, stats: WINDOW, ch: i32, curse_floor: char, game_objects: &HashMap<String, Tile>, player: &mut Player) {
        if ch == KEY_UP {
            // Determine the game object key of the tile above the cursor
            let key = (player.y-1).to_string() + "x" + &(player.x).to_string();
            // Check if the tile above the cursor is a wall or floor
            match game_objects.get(&key) {
                Some(tile) => {
                    if tile.c == curse_floor{
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
                    if tile.c == curse_floor{
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
                    if tile.c == curse_floor {
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
                    if tile.c == curse_floor {
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
}

impl Curses {
    pub fn new (height: i32, width: i32, curse_wall: char, curse_floor: char, curse_player: char) -> Curses {
        let curses = Curses {
            height: height,
            width: width,
            curse_wall: curse_wall,
            curse_floor: curse_floor,
            curse_player: curse_player,
            curse_color_wall: 1,
            curse_color_floor: 2,
            curse_color_player: 3
        };
        initscr();
        raw();
        keypad(stdscr(), true);
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        noecho();
        // Enable curses colors
        start_color();
        use_default_colors();
        // Create custom color pairs for: player, floors, and walls
        init_pair(curses.curse_color_wall, 57, 234);
        init_pair(curses.curse_color_floor, 60, 0);
        init_pair(curses.curse_color_player, 35, 0);
        // BUG if refresh is not run at least once, no windows refresh works (uhh?)
        refresh();
        curses
    }
    fn draw_player (&self, win: WINDOW) {
        mvwaddch(win, self.height/2, self.width/2, self.curse_player as chtype);
    }
    // Returns a new ncurses window with specific size (main map window)
    fn make_window(&self) -> WINDOW {
        let win = newwin(self.height, self.width, 0, 0);
        box_(win, 0, 0);
        wrefresh(win);
        win
    }

    // Returns ncurses window to the right of main window (little side window)
    fn make_stats_windows(&self) -> WINDOW {
        let win = newwin(5, 15, 2, self.width + 1);
        box_(win, 0, 0);
        wrefresh(win);
        win
    }
    // Create new window to get filename string from user, validate string, return filename string
    // Used for both map generation and map loading: uses check_exists to switch between each use case
    fn get_map_filename(&self, check_exists: bool) -> String {
        let win = self.make_window();
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
    fn get_map_size(&self) -> i32 {
        let win = self.make_window();
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
    // Menu gives the user options to generate a new map or load a saved map
    pub fn main_menu (&self) -> Map {
        let win = self.make_window();
        let mut ch: i32;
        //let game_objects: HashMap<String, maps::Tile>;
        let map: Map;
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
                let path: String = self.get_map_filename(true);
                let size: i32 = self.get_map_size();
                map = Map::new(size, size, '#', '.', 'P');
                Map::save_map(&path, &map, true);
                break;
            } else if ch == 'l' as i32 || ch == 'L' as i32 {
                // Get filename from user, try loading the map (should probably check for errors, but won't)
                let path: String = self.get_map_filename(false);
                map = Map::load_map(&path, true);
                //game_objects = maps::load_map(&path, true);
                break;
            } else if ch == 'q' as i32 || ch == 'Q' as i32 {
                // Exit program
                Curses::exit(0);
            }
        }
        endwin();
        map
    }
    // Draws the map. Only draws tiles within players view window, ignores rest of map.
    fn draw_map(&self, win: WINDOW, game_objects: &HashMap<String, Tile>, player: &Player) {
        let mid_y = self.height / 2;
        let mid_x = self.width / 2;
        // Use players position and map size to determine the loop size
        for ty in player.y-mid_y..player.y+mid_y {
            for tx in player.x-mid_x..player.x+mid_x {
                // Hashmap stores keys as string "Ypos" + "x" + "Xpos"
                let key = ty.to_string() + "x" + &tx.to_string();
                // Check if key exists to avoid crashes (this might let a corrupted map run)
                match game_objects.get(&key) {
                    Some(_tile_name) => {
                        // Draw and color each map tile relative to player position
                        wattr_on(win, self.color_tile(game_objects[&key].c));
                        mvwaddch(win, game_objects[&key].y-player.y+mid_y, game_objects[&key].x-player.x+mid_x, game_objects[&key].c as chtype);
                        wattr_off(win, self.color_tile(game_objects[&key].c));
                    },
                    None => {}
                }
            }
        }
        // Draw map border and player on top of the map
        box_(win, 0,0);
        wattr_on(win, COLOR_PAIR(self.curse_color_player));
        self.draw_player(win);
        wattr_off(win, COLOR_PAIR(self.curse_color_player));
    }
    // Draws a window to the right of screen with position data
    fn draw_stats (&self, win: WINDOW, player: &Player) {
        let posx = String::from("pos_x: ") + &player.x.to_string();
        let posy = String::from("pos_y: ") + &player.y.to_string();
        wclear(win);
        mvwaddstr(win, 1, 1, &posx);
        mvwaddstr(win, 2, 1, &posy);
        box_(win, 0, 0);
        wrefresh(win);
    }
    // Get player input, draw map view, main loop
    pub fn play_map(&self, game_objects: &HashMap<String, Tile>) {
        let win = self.make_window();
        let stats = self.make_stats_windows();
        nodelay(win, true);
        // Create player and add offset position
        let mut player = Player {y: self.height/2, x: self.width/2, c: self.curse_player};
        self.draw_map(win, &game_objects, &player);
        wrefresh(win);
        let mut ch = 0;
        while ch != 'q' as i32 {
            ch = getch(); // Reduce flickering by wgetch in separate window?
            flushinp();
            // Force map to always be same size, no matter if window resized
            wresize(win, self.height, self.width);
            Player::get_keyboard_input(win, stats, ch, self.curse_floor, &game_objects, &mut player);
            self.draw_map(win, &game_objects, &player);
            self.draw_stats(stats, &player);
            wnoutrefresh(win);
            thread::sleep(Duration::from_millis(30)); // reduces screen flicker a little, slows player input
            wclear(win);
            doupdate();
        }
        Curses::exit(0);
    }
    // Is used to color floors, walls, and players different set colors (color pairs must be created first)
    fn color_tile(&self, c: char) -> attr_t {
        if c == self.curse_wall {
            COLOR_PAIR(self.curse_color_wall)
        } else if c == self.curse_floor {
            COLOR_PAIR(self.curse_color_floor)
        } else {
            COLOR_PAIR(self.curse_color_player)
        }
     }
    // This exit function tries its best to fix curses terminal bugs before terminating program
    pub fn exit(exit_code: i32) {
        clear();
        curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
        nodelay(stdscr(), false);
        flushinp();
        echo();
        endwin();
        std::process::exit(exit_code);
    }

}


impl Tile {
    pub fn new(y: i32, x: i32, c: char, neighbors: Vec<String>) -> Tile {
        Tile { y: y, x: x, c: c, neighbors: neighbors }
    }
    // Create new tile key string, xy coordinate with separator
    pub fn get_tile_key(&self) -> String {
        let ty = &self.y.to_string();
        let tx = &self.x.to_string();
        let sep = String::from("x");
        let mut s = String::new();
        s.push_str(&ty);
        s.push_str(&sep);
        s.push_str(&tx);
        s
    }
    // Calculate distance between tiles for v-regions in gen_map
    pub fn distance(v: &Tile, t: &Tile) -> i32 {
        let distance = (v.y - t.y).abs() + (v.x - t.x).abs();
        distance
    }

    // Calculate distance between tiles for v-regions in gen_map
    pub fn distance_slow(v: &Tile, t: &Tile) -> i32 {
        let y = (v.y - t.y).pow(2) as f64;
        let x = (v.x - t.x).pow(2) as f64;
        let distance = (y + x).sqrt() as i32;
        distance
    }
}

impl Map {
    // Gen map is used to create a variable sized map using voronoi regions (can be very slow)
    pub fn new(sizey: i32, sizex: i32, map_wall: char, map_floor: char, map_player: char) -> Map {
        // Random numbers
        let mut rng = rand::thread_rng();
        // Create a new set of game_objects for final results
        let mut go = HashMap::new();
        // Generate v-regions based on size of map (# of regions matches maps.py YOU LIED HERE, HALF AS MANY!)
        let mut v_regions = Vec::new();
        let number_of_regions = rng.gen_range((sizey + sizex)/2, (sizey + sizex)*2);
        for v in 0..number_of_regions {
            let y = rng.gen_range(0, sizey);
            let x = rng.gen_range(0, sizex);
            let tile_type = rng.gen_range(0, 2);
            // Generate a string name for v-regions, must survive in game_objects json: v# is key
            let tv = v.to_string();
            let mut vkey = String::from("v");
            vkey.push_str(&tv);
            if tile_type == 0 {
                v_regions.push(Tile::new(y, x, map_wall, Vec::new()));
                go.insert(vkey, Tile::new(y, x, map_wall, Vec::new()));
            } else {
                v_regions.push(Tile::new(y, x, map_floor, Vec::new()));
                go.insert(vkey, Tile::new(y, x, map_floor, Vec::new()));
            }
        }
        // Add extra region for spawn location
        v_regions.push(Tile::new(12, 35, map_floor, Vec::new()));
        // Connect vregions through triangulation (update v-region tile closest neighbors)
        for v in 0..number_of_regions {
            let mut closest1 = String::new();
            let mut distance1 = 1000;
            let mut closest2 = String::new();
            let mut distance2 = 1000;
            // Create vkey
            let vkey_num = v.to_string();
            let mut vkey = String::from("v");
            vkey.push_str(&vkey_num);
            // Check each region for closest neighbors
            // Need to enforce angle sizes later...
            for vc in 0..number_of_regions {
                // Skip if self, a points closest point can't be itself
                if v == vc {
                    continue;
                }
                // Create key for closest compare
                let mut vckey = String::from("v");
                vckey.push_str(&vc.to_string());
                // calc distance
                let vcdist = Tile::distance_slow(&go[&vkey], &go[&vckey]);
                // Check if distance is closer, shuffle down the largest distance
                if vcdist < distance1 {
                    if distance1 < distance2 {
                        distance2 = distance1.clone();
                        closest2 = closest1.clone();
                    }
                    distance1 = vcdist.clone();
                    closest1 = vckey.clone();
                } else if vcdist < distance2 {
                    distance2 = distance1.clone();
                    closest2 = closest1.clone();
                }
            }
            // Add two closest neighbors to go map
            let y = go[&vkey].y;
            let x = go[&vkey].x;
            let c = go[&vkey].c;
            let mut neighbors = Vec::new();
            neighbors.push(closest1.clone());
            neighbors.push(closest2.clone());
            let t = Tile::new(y, x, c, neighbors);
            go.insert(t.get_tile_key(), t);
        }
        // Need to do multiple passes before storing in go!
        // You need to modify not only the tile you are working on, BUT ALL THE NEIGHBORS MUST BE UPDATED WITH THIS TILE
        // Maybe we get the 5 closest points and make triangles that force specific angle sizes
        // This could be done by making an additional pass over the neighbors and eliminating any that have bad characteristics
        // This could also lead to a situation where, if a point cannot be used in any valid triangles, we destroy it
        //take a break from this for now, lost motivatio    n


        let mapsize = Tile::new(sizey, sizex, '$', Vec::new());
        let player = Tile::new(0, 0, map_player, Vec::new());
        // Additional metadata to save in case another programs needs to know the map size
        go.insert(String::from("mapsize"), mapsize);
        go.insert(String::from("player"), player);
        // Manually add each v-region Tile
        // Hashmap for storing tiles and map data
        let mut game_objects = HashMap::new();
        // Generate temporary tiles
        for y in 0..sizey {
            for x in 0..sizex {
                let t = Tile::new(y, x, map_floor, Vec::new());
                game_objects.insert(t.get_tile_key(), t);
            }
        }
        // Create voronoi regions, convert tiles in game_objects to closest v-region tile type
        // This loop plus the distance calc might be slowest part of gen_map
        for k in game_objects.keys() {
            let mut closest: usize = 0;
            for i in 0..number_of_regions+1 { // Add +1 extra region for spawn location
                // Distance calc code
                let cur: usize = i as usize;
                let diff = Tile::distance(&v_regions[cur], &game_objects[k]);
                let old_diff = Tile::distance(&v_regions[closest], &game_objects[k]);
                if diff < old_diff {
                    closest = cur;
                }
            }
            // Add walls on edges because why not
            let ttype: char;
            if game_objects[k].y <= 0 || game_objects[k].y >= sizey - 1 || game_objects[k].x <= 0 || game_objects[k].x >= sizex - 1 {
                ttype = map_wall;
            } else {
                ttype = v_regions[closest].c;
            }
            let t = Tile {y: game_objects[k].y, x: game_objects[k].x, c: ttype, neighbors: Vec::new()};
            // Modifying game_objects tiles is a pain, so we make the changes to a mirror data structure called go
            go.insert(k.to_string(), t);
        }
        // return the modified map data structure
        //go
        let map: Map = Map {
            map_wall: '#',
            map_floor: '.',
            map_player: 'p',
            map_game_objects: go
        };
        map
    }
    // Opens a file for reading to decompress, deserialize, and store as hashmap
    pub fn load_map(filename: &str, compression: bool) -> Map {
         //s = String::new();
        let mut f = File::open(filename).expect("Unable to open file");
        let mut s = String::new();
        if compression {
            s = Map::decompress(&f);
        } else {
            f.read_to_string(&mut s).unwrap();
        }
        //GzDecoder::new(f).read_to_string(&mut s).unwrap();
        let game_objects: HashMap<String, Tile> = serde_json::from_str(&s).unwrap();
        let map: Map = Map {
            map_wall: '#',
            map_floor: '.',
            map_player: 'p',
            map_game_objects: game_objects
        };
        map
    }
    // Serialize hashmap into string, open a file for writing, write to file with compressed bufwriter
    pub fn save_map (filename: &str, map: &Map, compression: bool) {
        let serialized = serde_json::to_string(&map.map_game_objects).unwrap();
        let f = File::create(filename).expect("Unable to create file");
        let enc: flate2::write::GzEncoder<std::fs::File>;
        // if compression enabled, gzip here
        if compression {
            enc = Map::compress(f);
            let mut buf = BufWriter::new(enc);
            buf.write_all(serialized.as_bytes()).expect("Unable to write data");
        } else {
            //enc = f;
            let mut buf = BufWriter::new(f);
            buf.write_all(serialized.as_bytes()).expect("Unable to write data");
        }
    }
    // Write wrapper to compress file, return encoder file
    pub fn compress(file: File) -> flate2::write::GzEncoder<std::fs::File>  {
        let enc = GzEncoder::new(file, Compression::default());
        enc
    }

    // Write wrapper to decompress file, return string
    pub fn decompress(f: &std::fs::File) -> String{
        let mut s = String::new();
        GzDecoder::new(f).read_to_string(&mut s).unwrap();
        s
    }
}
