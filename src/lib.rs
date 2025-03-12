#![no_std]

// There needs to be a game manager object, player object, and debris object.
// The game manager needs to spawn a player, and spawn debris every few seconds at a random row with a random color/speed.
// It also needs to keep a score.
// The player can move up and down at a consistent speed, and can't stop whenever it starts moving for the first time.
// The player gets a point when a piece of debris flies past, e.g. the debris reaches the player's column - 1.
// The debris object gets destroyed once it reaches the left of the screen.
// The player loses when they hit a debris, the sprite turning into a * and pausing the game.
// There should be a header that displays the score and a game over message when the player loses.
// When the player loses, they can press R to restart the game.
// For audio, there can be a sound effect when the player moves, when they get a point, and when they lose. Maybe
// when they restart the game as well.

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH,
};

use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    marker::Copy,
    prelude::rust_2024::derive,
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SpaceDebrisGame {
    player: Player,
    debris: Debris,
    score: u32
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Player {
    col: usize,
    row: usize,
    dy: isize
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Debris {
    col: usize,
    row: usize,
    dx: isize,
    color: Color
}

pub fn safe_add<const LIMIT: usize>(a: usize, b: usize) -> usize {
    (a + b).mod_floor(&LIMIT)
}

pub fn add1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, 1)
}

pub fn sub1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, LIMIT - 1)
}

impl Default for SpaceDebrisGame {
    fn default() -> Self {
        Self {
            player: Player::default(),
            debris: Debris::default(),
            score: 0
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            col: BUFFER_WIDTH / 4,
            row: BUFFER_HEIGHT / 2,
            dy: 0
        }
    }
}

impl Default for Debris {
    fn default() -> Self {
        Self {
            col: BUFFER_WIDTH / 2,
            row: BUFFER_HEIGHT / 2,
            dx: 1,
            color: Color::White
        }
    }
}

impl Player {
    pub fn tick(&mut self) {
        self.clear_current();
        self.update_location();
        self.draw_current();
    }

    fn clear_current(&self) {
        plot(' ', self.col, self.row, ColorCode::new(Color::Black, Color::Black));
    }

    fn update_location(&mut self) {
        if self.dy < 0 {
            self.row = sub1::<BUFFER_HEIGHT>(self.row);
        } else if self.dy > 0 {
            self.row = add1::<BUFFER_HEIGHT>(self.row);
        }
    }

    fn draw_current(&self) {
        plot(
            '>',
            self.col,
            self.row,
            ColorCode::new(Color::White, Color::Black),
        );
    }

    pub fn key(&mut self, key: DecodedKey) {
        if let DecodedKey::RawKey(code) = key {
            self.handle_raw(code);
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowUp => {
                self.dy = -1;
            }
            KeyCode::ArrowDown => {
                self.dy = 1;
            }
            _ => {}
        }
    }
}

impl Debris {
    pub fn tick(&mut self) {
        self.clear_current();
        self.update_location();
        self.draw_current();
    }

    fn clear_current(&self) {
        plot(' ', self.col, self.row, ColorCode::new(Color::Black, Color::Black));
    }

    fn update_location(&mut self) {
        self.col = add1::<BUFFER_WIDTH>(self.col);
    }

    fn draw_current(&self) {
        plot(
            '*',
            self.col,
            self.row,
            ColorCode::new(self.color, Color::Black),
        );
    }
}
