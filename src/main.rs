/*
A simple 2048 clone.
Copyright (C) 2016  Gregory Katz

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

extern crate rand;
extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate piston;
extern crate graphics;

use piston_window::*;
use std::{process, env};
use rand::distributions::{IndependentSample, Range};
use graphics::character::CharacterCache;


#[derive(Default)]
struct Board {
    data: [u64; 16],
    score: u64,
}

const ROUNDNESS:f64 = 7.0;
const BOARD_START_Y:f64 = 150.0;
const BOARD_START_X:f64 = 10.0;
const WINDOW_WIDTH:u32 = 430;
const WINDOW_HEIGHT:u32 = 600;

enum Direction {
    Left,
    Right,
    Down,
    Up,
}

enum UserInput {
    Move(Direction),
    Quit,
    Reset,
    About,
}

impl Board {
    fn add_random(&mut self) {
        // Find a random empty tile and add a 2 or a 4
        let mut rng = rand::thread_rng();
        let range = Range::new(0, 16);
        let tile = range.ind_sample(&mut rng);
        match self.data[tile] {
            0 => self.data[tile] = two_or_four() as u64,
            _ => self.add_random(),
        }
    }

    fn check_loss(&self) -> bool {
        // Checks whether the board is in a loss condition
        // Array of neighbors
        let mut ns: [Option<usize>; 4] = [None, None, None, None];
        for (i, elem) in self.data.iter().enumerate() {

            // The user can always make another move if there's an empty tile
            if *elem == 0 {
                return false;
            }

            // Calculate each tile's neighbors
            // This could be done statically for a 4x4 board but
            // I may want to expand the game board some day
            ns[0] = if (i + 1) % 4 == 1 { None } else { Some(i - 1) };
            ns[1] = if i % 4 == 3 { None } else { Some(i + 1) };
            ns[2] = if i < 4 { None } else { Some(i - 4) };
            ns[3] = if i > 11 { None } else { Some(i + 4) };

            // Iterate over the neighbors and check if a
            // combination is possible 
            // I suppose the last row need not be checked, but I'm lazy
            for e in &ns {
                if let Some(x) = *e {
                    if *elem == self.data[x] {
                        return false;
                    }
                };
            }
        }
        true
    }

    fn player_move(&mut self, dir: Direction) ->
        std::result::Result<(), &str> {
        // Main game board logic
        let old = self.data;

        // Go row by row. The direction of the row depends on the user's input
        'rowloop: for base in 0..4usize {
            // Store a closure that calcuates where the next tile down is based
            // on input direction
            // TODO Use function pointers instead

            let down: Box<Fn(usize) -> usize> = match dir {
                Direction::Up =>
                    Box::new(|x: usize| base + (4usize * x)),
                Direction::Down =>
                    Box::new(|x: usize| base + 12 - (4usize * x)),
                Direction::Left =>
                    Box::new(|x: usize| (base * 4) + x),
                Direction::Right =>
                    Box::new(|x: usize| (base * 4) + 3 - x),
            };

            // Bubble up zeros
            for _ in 0..3 {
                if self.data[down(0)] == 0 {
                    self.data.swap(down(0), down(1));
                    self.data.swap(down(1), down(2));
                    self.data.swap(down(2), down(3));
                }
            }
            for _ in 0..2 {
                if self.data[down(1)] == 0 {
                    self.data.swap(down(1), down(2));
                    self.data.swap(down(2), down(3));
                }
            }
            for _ in 0..1 {
                if self.data[down(2)] == 0 {
                    self.data.swap(down(2), down(3));
                }
            }

            // Consolidate equal tiles
            if self.data[down(0)] == self.data[down(1)] &&
                self.data[down(2)] == self.data[down(3)] {
                    self.score = self.score + self.data[down(0)] * 2;
                    self.score = self.score + self.data[down(2)] * 2;
                    self.data[down(0)] = self.data[down(0)] * 2;
                    self.data[down(1)] = self.data[down(2)] * 2;
                    self.data[down(2)] = 0;
                    self.data[down(3)] = 0;
                    continue 'rowloop;
            }

            if self.data[down(0)] == self.data[down(1)] &&
                self.data[down(2)] != self.data[down(3)] {
                    self.score = self.score + self.data[down(0)] * 2;
                    self.data[down(0)] = self.data[down(0)] * 2;
                    self.data[down(1)] = self.data[down(2)];
                    self.data[down(2)] = self.data[down(3)];
                    self.data[down(3)] = 0;
                    continue 'rowloop;
            }

            if self.data[down(0)] != self.data[down(1)] &&
                self.data[down(1)] == self.data[down(2)] {
                    self.score = self.score + self.data[down(1)] * 2;
                    self.data[down(1)] = self.data[down(1)] * 2;
                    self.data[down(2)] = self.data[down(3)];
                    self.data[down(3)] = 0;
                    continue 'rowloop;
            }

            if self.data[down(2)] != self.data[down(1)] &&
               self.data[down(2)] == self.data[down(3)] {
                if self.data[down(3)] == 0 {
                    continue 'rowloop;
                }
                   self.score = self.score + self.data[down(2)] * 2;
                   self.data[down(2)] = self.data[down(2)] * 2;
                   self.data[down(3)] = 0;
                   continue 'rowloop;
            }
        }

        // TODO Detect that the board changed without copying the board
        if old != self.data {
            self.add_random();
            Ok(())
        } else { Err("Player played an illegal move") }
    }

    #[allow(dead_code)]
    fn print(&self) {
        // Prints the board to terminal for debugging
        println!("{}   {}   {}   {}",
                 self.data[0],
                 self.data[1],
                 self.data[2],
                 self.data[3]);
        println!("{}   {}   {}   {}",
                 self.data[4],
                 self.data[5],
                 self.data[6],
                 self.data[7]);
        println!("{}   {}   {}   {}",
                 self.data[8],
                 self.data[9],
                 self.data[10],
                 self.data[11]);
        println!("{}   {}   {}   {}",
                 self.data[12],
                 self.data[13],
                 self.data[14],
                 self.data[15]);
        println!("");
    }
}

fn get_user_input(event: Button) -> Option<UserInput> {
    // Returns a cardinal direction corresponding to user input or none
    if let Button::Keyboard(key) = event {
        match key {
            Key::Left | Key::A => Some(UserInput::Move(Direction::Left)),
            Key::Up | Key::W => Some(UserInput::Move(Direction::Up)),
            Key::Down | Key::S => Some(UserInput::Move(Direction::Down)),
            Key::Right | Key::D => Some(UserInput::Move(Direction::Right)),
            Key::Q | Key::Escape => Some(UserInput::Quit),
            Key::R => Some(UserInput::Reset),
            Key::H => Some(UserInput::About),
            _ => None, // Unrecognized key. Silently ignore.
        }
    } else { None } // Not a keyboard event. Silently ignore.
}

fn get_tile_color(value: u64) -> types::Color {
    // TODO add more colors for higher tiles
    match value {
            0 => [0.80, 0.75, 0.70, 1.0],
            2 => [0.93, 0.89, 0.85, 1.0],
            4 => [0.93, 0.88, 0.78, 1.0],
            8 => [0.95, 0.69, 0.47, 1.0],
            16 => [0.96, 0.58, 0.39, 1.0],
            32 => [0.96, 0.49, 0.37, 1.0],
            64 => [0.96, 0.36, 0.23, 1.0],
            128 => [0.93, 0.80, 0.44, 1.0],
            256 => [0.93, 0.80, 0.37, 1.0],
            512 => [0.93, 0.78, 0.31, 1.0],
            1024 => [0.93, 0.80, 0.38, 1.0],
            2048 => [0.93, 0.76, 0.18, 1.0],
            _ => [0.0, 0.0, 0.0, 1.0],
        }
}

fn render_about(c: &graphics::Context,
                g: &mut G2d,
                glyphs: &mut Glyphs) {
    let rect =
        graphics::rectangle::Rectangle::new_round([0.0, 0.0, 0.0, 0.5],
                                                  0.0);
    rect.draw([0.0, 0.0, WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64],
              &c.draw_state,
              c.transform, g);
    let mut cr = 0.0;
    let text = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 12);
    let lines = ["r2048 Copyright (C) 2016  Gregory Katz",
                 "This program comes with ABSOLUTELY NO WARRANTY;",
                 "for details see www.gnu.org/licenses/gpl.txt",
                 "This is free software, and you are welcome to redistribute it",
                 "under certain conditions;",
                 "See www.gnu.org/licenses/gpl.txt for details.",
                 "",
                 "This program uses the Clear Sans font",
                 "distributed under the terms of the Apache 2.0 license",
                 "available at http://www.apache.org/licenses/LICENSE-2.0",];
    for line in lines.iter() {
        let width = glyphs.width(12, line);
        let transform = c.transform
            .trans((WINDOW_WIDTH as f64 / 2.0 ) - (width / 2.0),
                   WINDOW_HEIGHT as f64 / 2.0  + cr );
        text.draw(line,
                  glyphs,
                  &c.draw_state,
                  transform,
                  g);
        cr = cr + 14.0;
    }    
}

fn render_loss_screen(c: &graphics::Context,
                g: &mut G2d,
                glyphs: &mut Glyphs) {

    let rect =
        graphics::rectangle::Rectangle::new_round([0.0, 0.0, 0.0, 0.5],
                                                  0.0);
    rect.draw([0.0, 0.0, WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64],
              &c.draw_state,
              c.transform, g);
    
    let text = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 30);
    let width = glyphs.width(30, "You lost!");
    let transform = c.transform
        .trans((WINDOW_WIDTH as f64 / 2.0 ) - (width / 2.0),
               WINDOW_HEIGHT as f64 / 2.0 );
    text.draw("You lost!",
              glyphs,
              &c.draw_state,
              transform,
              g);
    let width = glyphs.width(30, "Press R to start a new game.");
    let transform = c.transform
        .trans((WINDOW_WIDTH as f64 / 2.0 ) - (width / 2.0),
               (WINDOW_HEIGHT as f64 / 2.0) + 32.0 );
    text.draw("Press R to start a new game.",
              glyphs,
              &c.draw_state,
              transform,
              g);    
}

fn render_board(board: &Board,
                c: &graphics::Context,
                g: &mut G2d,
                glyphs: &mut Glyphs) {
    // Clear the window
    clear([1.0; 4], g);

    // Draw stuff above the game board
    // Game title

    let rect =
        graphics::rectangle::Rectangle::new_round(get_tile_color(2048),
                                                  ROUNDNESS);
    rect.draw([BOARD_START_X, 5.0, 128.0, 128.0],
              &c.draw_state,
              c.transform, g);
    
    let transform = c.transform
        .trans(BOARD_START_X + 3.0, 88.0);
    let text = text::Text::new_color([1.0, 1.0, 1.0, 1.0], 52);
    text.draw("2048",
              glyphs,
              &c.draw_state,
              transform,
              g);

    let transform = c.transform
        .trans(BOARD_START_X + 250.0, 40.0);
    let text = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 20);
    text.draw("SCORE:",
              glyphs,
              &c.draw_state,
              transform,
              g);
    text.draw(&format!("{}", board.score),
              glyphs,
              &c.draw_state,
              transform.trans(0.0, 22.0),
              g);

    let transform = c.transform
        .trans(BOARD_START_X + 250.0, 40.0);
    let text = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 20);
    text.draw("SCORE:",
              glyphs,
              &c.draw_state,
              transform,
              g);
    text.draw(&format!("{}", board.score),
              glyphs,
              &c.draw_state,
              transform.trans(0.0, 22.0),
              g);
    
    let text = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 16);
    let transform = transform.trans(-80.0, 22.0);
    text.draw("Press R to reset the game",
              glyphs,
              &c.draw_state,
              transform.trans(0.0, 40.0),
              g);
    text.draw("Press H for copyright info",
              glyphs,
              &c.draw_state,
              transform.trans(0.0, 58.0),
              g);

    // Draw game board background
    let board_background =
        graphics::rectangle::Rectangle::new_round([0.74, 0.71, 0.65, 1.0],
                                                  ROUNDNESS);
    board_background.draw([BOARD_START_X, BOARD_START_Y, 410.0, 410.0],
                          &c.draw_state, c.transform, g);
    
    // Iterate over the game board and draw the tiles
    for (i, elem) in board.data.iter().enumerate() {

        // Scale factor for text to fit longer numbers in tile
        let font_size = match *elem {
            2 | 4 | 8 => 60,
            16 | 32 | 64 => 52,
            128 | 256 | 512 => 46,
            1024 | 2048 | 4096 | 8192 => 34,
            _ => 28,
        };
        
        // Calculate the tile's origin point
        let tile_x = ((i % 4) as f64 * 100.0) + 10.0 + BOARD_START_X;
        let tile_y = ((i / 4) as f64 * 100.0) + 10.0 + BOARD_START_Y;
        
        // Draw the tile
        let rect =
            graphics::rectangle::Rectangle::new_round(get_tile_color(*elem),
                                                      ROUNDNESS);
        rect.draw([tile_x, tile_y, 90.0, 90.0], &c.draw_state, c.transform, g);

        // Skip text draw if the tile's empty 
        if *elem == 0 { continue; }

        // Draw the tile's number in the center of the tile
        // TODO Use static strs instead of referencing allocated strings
        let width = glyphs.width(font_size, &format!("{}", elem));
        // Horizontal dentering works great but vertical is a hack
        // I have no idea why we're dividing by 3, but it looks ok
        let transform = c.transform
            .trans(tile_x + (45.0 - (width / 2.0)),
                   tile_y + 45.0 + (font_size as f64 / 3.0));
        // Text is black for 2 and 4, white otherwise
        let text = text::Text::new_color(
            match *elem {
                2 | 4 => [0.50, 0.48, 0.45, 1.0],
                _ => [1.0, 1.0, 1.0, 1.0],
            },
            font_size);
        text.draw(&format!("{}", elem),
                  glyphs,
                  &c.draw_state,
                  transform,
                  g);
    }
}

fn two_or_four() -> u8 {
    // Returns a 2 90% of the time and 4 10% of the time
    let mut rng = rand::thread_rng();
    let range = Range::new(0, 10);
    match range.ind_sample(&mut rng) {
        0 => 4,
        _ => 2,
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    // Initalize an empty board
    let mut board = Board::default();

    // Add a 2 or 4 in a random spot
    board.add_random();

    // For development use only 
    if args.len() > 1 {
        if args[1] == "-devel_board" || args[1] == "-d" {
            board.data = [2, 4, 8, 16, 32, 64, 128, 256,
                          512, 1024, 2048, 4096, 8192, 16384,
                          0, 0];
            }
    }

    // Create the main window
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("r2048", (WINDOW_WIDTH, WINDOW_HEIGHT))
        .exit_on_esc(false)
        .opengl(opengl)
        .build()
        .expect("Unable to display game window.");
    window.set_should_close(false);
    // Load font
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .expect("Could not locate assets folder");
    let font = &assets.join("ClearSans-Bold.ttf");
    let factory = window.factory.clone();
    let mut glyphs = Glyphs::new(font, factory)
        .expect("Could not convert font to glyphs");
    let mut draw = true;
    let mut about = false;
    
    // Enter game loop
    while let Some(e) = window.next() {
        // Draw the board if a draw flag is set
        if draw {
            window.draw_2d(&e, |c, g| {
                render_board(&board, &c, g, &mut glyphs);
                // Show a loss screen if the player lost
                if board.check_loss() {
                    render_loss_screen(&c, g, &mut glyphs);
                }
                if about {
                    render_about(&c, g, &mut glyphs);
                }
                draw = false;
            });
        }
    
        // Check if there's user input
        if let Some(event) = e.press_args() {
            // If the about dialog is showing, ignore other input
            // and wait for the user to hit escape
            if about {
                match get_user_input(event) {
                    Some(UserInput::Quit) => {
                        about = false;
                        draw = true;
                        continue;
                    },
                    _ => continue, 
                }
            }
            
            match get_user_input(event) {
                // If player makes a vaid move, execute the move
                // and set redraw flag
                Some(UserInput::Move(dir)) =>
                    if board.player_move(dir).is_ok() { draw = true },
                // Exit if player hits Q
                Some(UserInput::Quit) => process::exit(0),
                // Reset the board if player hits R
                Some(UserInput::Reset) => {
                    draw = true;
                    board = Board::default();
                    board.add_random();
                },
                Some(UserInput::About) => {
                    about = true;
                    draw = true;
                },
                None => {}
            };
        }
    }
}
