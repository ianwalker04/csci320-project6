#![no_std]

use num::Integer;
use heapless::Vec;
use oorandom::{self, Rand32};
use pc_keyboard::{DecodedKey, KeyCode};
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

#[derive(Clone, Eq, PartialEq)]
pub struct SpaceDebrisGame {
    player: Player,
    debris: Vec<Debris, 30>,
    score: u32,
    spawn_countdown: u32,
    seed_count: u32
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
    dx: usize,
    dx_tick: usize,
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
            debris: Vec::new(),
            score: 0,
            spawn_countdown: 5,
            seed_count: 0
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
    pub fn new(num: u32) -> Self {
        let mut rng: Rand32 = Rand32::new(num.into());
        Self {
            col: BUFFER_WIDTH - 1,
            row: rng.rand_range(2..BUFFER_HEIGHT as u32) as usize,
            dx: rng.rand_range(1..5) as usize,
            dx_tick: 0,
            color: DEBRIS_COLORS[rng.rand_range(0..13) as usize],
            debris_status: DebrisStatus::Normal
        }
    }
}

impl SpaceDebrisGame {
    pub fn update(&mut self) {
        self.seed_count += 1;
        if let Some(event) = self.player.tick() {
            match event {
                GameStatus::GameRunning => {},
                GameStatus::GameOver => {
                    self.player.dy = 0;
                    self.display_game_over();
                }
            }
        }
        let mut deleted_debris: Vec<usize, 30> = Vec::<usize, 30>::new();
        for i in 0..self.debris.len() {
            if let Some(event) = self.debris[i].tick(&mut self.player) {
                match event {
                    DebrisStatus::ScorePoint => self.increment_score(),
                    DebrisStatus::Destroy => {
                        let _ = deleted_debris.push(i);
                    },
                    DebrisStatus::Normal => {}
                }
            }
        }
        for &debris in deleted_debris.iter().rev() {
            self.debris.remove(debris);
        }
        self.create_debris();
    }

    pub fn initialize(&mut self) {
        self.display_score();
    }

    pub fn create_debris(&mut self) {
        self.seed_count += 1;
        if self.spawn_countdown == 0 {
            let _ = self.debris.push(Debris::new(self.seed_count));
            self.spawn_countdown = 5;
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
        if self.player.game_status != GameStatus::GameOver {
            self.score += 1;
            self.display_score();
        }
    }

    pub fn display_score(&self) {
        let header_color: ColorCode = ColorCode::new(Color::White, Color::Black);
        let score_text: &str = "Score: ";
        clear_row(0, Color::Black);
        plot_str(score_text, 0, 0, header_color);
        plot_num(self.score as isize, score_text.len(), 0, header_color);
    }

    pub fn display_game_over(&self) {
        let header_color_score: ColorCode = ColorCode::new(Color::White, Color::Black);
        let header_color_gameover: ColorCode = ColorCode::new(Color::Red, Color::Black);
        let final_score_text: &str = "Final Score: ";
        let game_over_text: &str = "Game over! Press R to restart.";
        clear_row(0, Color::Black);
        plot_str(final_score_text, 0, 0, header_color_score);
        plot_num(self.score as isize, final_score_text.len(), 0, header_color_score);
        plot_str(game_over_text, 0, 1, header_color_gameover);
    }

    pub fn reset(&mut self) {
        self.player.game_status = GameStatus::GameRunning;
        self.player.clear_current();
        clear_row(0, Color::Black);
        clear_row(1, Color::Black);
        let mut deleted_debris: Vec<usize, 30> = Vec::<usize, 30>::new();
        for i in 0..self.debris.len() {
            let _ = deleted_debris.push(i);
            self.debris[i].clear_current();
        }
        for &debris in deleted_debris.iter().rev() {
            self.debris.remove(debris);
        }
        self.score = 0;
        self.player.row = BUFFER_HEIGHT / 2;
        self.player.col = BUFFER_WIDTH / 4;
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
            self.clear_current();
            return Some(DebrisStatus::Destroy);
        }
        return Some(DebrisStatus::Normal);
    }

    fn clear_current(&self) {
        plot(' ', self.col, self.row, ColorCode::new(Color::Black, Color::Black));
    }

    fn update_location(&mut self) {
        if self.dx_tick == 0 {
            self.col = sub1::<BUFFER_WIDTH>(self.col);
            self.dx_tick = self.dx;
        } else {
            self.dx_tick -= 1;
        }
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
