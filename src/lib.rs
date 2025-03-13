#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::println;
use pluggable_interrupt_os::vga_buffer::{
    plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, plot_str, plot_num, clear_row
};

use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    marker::Copy,
    prelude::rust_2024::derive,
};

const DEBRIS_COLORS: [Color; 13] = [Color::Blue, Color::Green, Color::Cyan, Color::Red, Color::Magenta,
                                    Color::LightGray, Color::LightBlue, Color::LightGreen, Color::LightCyan,
                                    Color::LightRed, Color::Pink, Color::Yellow, Color::White];

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameStatus {
    GameRunning,
    GameOver
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DebrisStatus {
    Normal,
    ScorePoint,
    Destroy
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SpaceDebrisGame {
    player: Player,
    debris: Debris,
    score: u32,
    spawn_countdown: u32
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Player {
    col: usize,
    row: usize,
    dy: isize,
    game_status: GameStatus
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Debris {
    col: usize,
    row: usize,
    dx: isize,
    color: Color,
    debris_status: DebrisStatus
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
            debris: Debris::new(),
            score: 0,
            spawn_countdown: 50
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            col: BUFFER_WIDTH / 4,
            row: BUFFER_HEIGHT / 2,
            dy: 0,
            game_status: GameStatus::GameRunning
        }
    }
}

impl Debris {
    pub fn new() -> Self {
        Self {
            col: BUFFER_WIDTH / 2,
            row: BUFFER_HEIGHT / 2,
            dx: 1,
            color: Color::White,
            debris_status: DebrisStatus::Normal
        }
    }
}

impl SpaceDebrisGame {
    pub fn update(&mut self) {
        if let Some(event) = self.player.tick() {
            match event {
                GameStatus::GameRunning => {},
                GameStatus::GameOver => {
                    self.player.dy = 0;
                    self.display_game_over();
                }
            }
        }
        if let Some(event) = self.debris.tick(&mut self.player) {
            match event {
                DebrisStatus::ScorePoint => self.increment_score(),
                DebrisStatus::Destroy => {
                    // Destroy debris
                    self.debris = Debris::new();
                },
                DebrisStatus::Normal => {}
            }
        }
        self.create_debris();
    }

    pub fn initialize(&mut self) {
        self.display_score();
    }

    pub fn create_debris(&mut self) {
        if self.spawn_countdown == 0 {
            // Spawn debris
        } else {
            self.spawn_countdown -= 1;
        }
    }

    pub fn key(&mut self, key: DecodedKey) {
        if self.player.game_status == GameStatus::GameOver {
            if key == DecodedKey::RawKey(KeyCode::R) || key == DecodedKey::Unicode('r') {
                self.reset();
            }
        }
        self.player.key(key);
    }

    pub fn increment_score(&mut self) {
        self.score += 1;
        self.display_score();
    }

    pub fn display_score(&self) {
        let header_color: ColorCode = ColorCode::new(Color::White, Color::Black);
        let score_text: &str = "Score: ";
        clear_row(0, Color::Black);
        plot_str(score_text, 0, 0, header_color);
        plot_num(self.score as isize, score_text.len() + 1, 0, header_color);
    }

    pub fn display_game_over(&self) {
        let header_color: ColorCode = ColorCode::new(Color::Red, Color::Black);
        let game_over_text: &str = "Game over! Press R to restart.";
        plot_str(game_over_text, 0, 1, header_color);
    }

    pub fn reset(&mut self) {
        self.player.game_status = GameStatus::GameRunning;
        clear_row(0, Color::Black);
        clear_row(1, Color::Black);
        self.score = 0;
        self.initialize();
    }
}

impl Player {
    pub fn tick(&mut self) -> Option<GameStatus> {
        self.clear_current();
        self.update_location();
        self.draw_current();
        if self.game_status == GameStatus::GameOver {
            return Some(GameStatus::GameOver);
        }
        return Some(GameStatus::GameRunning);
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

    fn collide(&mut self) {
        self.game_status = GameStatus::GameOver;
    }

    fn draw_current(&self) {
        if self.game_status == GameStatus::GameRunning {
            plot(
                '>',
                self.col,
                self.row,
                ColorCode::new(Color::White, Color::Black),
            );
        } else {
            plot(
                '*',
                self.col,
                self.row,
                ColorCode::new(Color::White, Color::Black),
            );
        }
    }

    pub fn key(&mut self, key: DecodedKey) {
        if let DecodedKey::RawKey(code) = key {
            self.handle_raw(code);
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        if self.game_status == GameStatus::GameRunning {
            match key {
                KeyCode::ArrowUp => {
                    self.dy = -1;
                },
                KeyCode::ArrowDown => {
                    self.dy = 1;
                },
                _ => {}
            }
        }
    }
}

impl Debris {
    pub fn tick(&mut self, player: &mut Player) -> Option<DebrisStatus> {
        self.clear_current();
        self.update_location();
        if self.col == player.col && self.row == player.row {
            player.collide();
        }
        self.draw_current();
        if self.col == 18 {
            return Some(DebrisStatus::ScorePoint);
        }
        if self.col == 0 {
            return Some(DebrisStatus::Destroy);
        }
        return Some(DebrisStatus::Normal);
    }

    fn clear_current(&self) {
        plot(' ', self.col, self.row, ColorCode::new(Color::Black, Color::Black));
    }

    fn update_location(&mut self) {
        self.col = sub1::<BUFFER_WIDTH>(self.col);
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
