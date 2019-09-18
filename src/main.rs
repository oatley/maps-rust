extern crate maps;

use maps::Map;
use maps::Curses;

// Initialize ncurses settings, create color pairs, start main menu, start map view
fn main() {
    // Move these to config later
    let window_height = 24;
    let window_width = 70;
    let map_wall = '#';
    let map_floor = '.';
    let map_player = 'P';
    // Create a new curses object
    let curses = Curses::new(window_height, window_width, map_wall, map_floor, map_player);
    // Start the main menu, user interface (gen or load)
    let map = curses.main_menu();
    // Start the main loop for viewing map
    curses.play_map(&map.map_game_objects);
}
