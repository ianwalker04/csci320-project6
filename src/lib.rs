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

// Level 3 Outline
// Start with a title screen, featuring a fancy SPACE JUNK title and the three difficulties.
// Player presses one of 1, 2, or 3 on the keyboard, corresponding to one of the three difficulties.
// As difficulty goes up, there is more debris and the debris is faster.
// When the player loses, a game over/high score screen is displayed showing the player's high score for all three
// difficulties. They can press R to go back to the title screen.

const DEBRIS_COLORS: [Color; 13] = [Color::Blue, Color::Green, Color::Cyan, Color::Red, Color::Magenta,
                                    Color::LightGray, Color::LightBlue, Color::LightGreen, Color::LightCyan,
                                    Color::LightRed, Color::Pink, Color::Yellow, Color::White];

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameStatus {
    GameRunning,
    GameOver
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Difficulty {
    Undefined,
    Cakewalk,
    RMT,
    Nightmare
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
    cw_high_score: u32,
    rmt_high_score: u32,
    n_high_score: u32,
    spawn_countdown: u32,
    seed_count: u32,
    difficulty: Difficulty
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
            cw_high_score: 0,
            rmt_high_score: 0,
            n_high_score: 0,
            spawn_countdown: 5,
            seed_count: 0,
            difficulty: Difficulty::Undefined
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

    pub fn display_title_screen(&mut self) {
        let header_color: ColorCode = ColorCode::new(Color::White, Color::Black);
        let text: &str = "";
        plot_str(text, 0, 0, header_color);
    }

    fn create_debris(&mut self) {
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

    fn increment_score(&mut self) {
        if self.player.game_status != GameStatus::GameOver {
            self.score += 1;
            self.display_score();
        }
    }

    fn display_score(&self) {
        let header_color: ColorCode = ColorCode::new(Color::White, Color::Black);
        let score_text: &str = "Score: ";
        clear_row(0, Color::Black);
        plot_str(score_text, 0, 0, header_color);
        plot_num(self.score as isize, score_text.len(), 0, header_color);
    }

    fn display_game_over(&self) {
        let header_color_score: ColorCode = ColorCode::new(Color::White, Color::Black);
        let header_color_gameover: ColorCode = ColorCode::new(Color::Red, Color::Black);
        let cw_score_text: &str = "High Score (Cakewalk): ";
        let rmt_score_text: &str = "High Score (Road Most Travelled): ";
        let n_score_text: &str = "High Score (Nightmare): ";
        let game_over_text: &str = "Game over! Press R to restart.";
        plot_str(cw_score_text, 0, 0, header_color_score);
        plot_num(self.cw_high_score as isize, cw_score_text.len(), 0, header_color_score);
        plot_str(rmt_score_text, 0, 1, header_color_score);
        plot_num(self.rmt_high_score as isize, rmt_score_text.len(), 1, header_color_score);
        plot_str(n_score_text, 0, 2, header_color_score);
        plot_num(self.n_high_score as isize, n_score_text.len(), 2, header_color_score);
        plot_str(game_over_text, 0, 3, header_color_gameover);
    }

    fn reset(&mut self) {
        self.player.game_status = GameStatus::GameRunning;
        self.player.clear_current();
        for i in 0..=3 {
            clear_row(i, Color::Black);
        }
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
    fn tick(&mut self) -> Option<GameStatus> {
        self.clear_current();
        self.update_location();
        self.draw_current();
        if self.game_status == GameStatus::GameOver {
            return Some(GameStatus::GameOver);
        }
        Some(GameStatus::GameRunning)
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
        }
    }

    fn key(&mut self, key: DecodedKey) {
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
    fn tick(&mut self, player: &mut Player) -> Option<DebrisStatus> {
        if player.game_status == GameStatus::GameRunning {
            self.clear_current();
            self.update_location();
            if self.col == player.col && self.row == player.row {
                player.collide();
            }
            self.draw_current(*player);
            if self.col == 18 {
                return Some(DebrisStatus::ScorePoint);
            }
            if self.col == 0 {
                self.clear_current();
                return Some(DebrisStatus::Destroy);
            }
            return Some(DebrisStatus::Normal);
        }
        self.clear_current();
        Some(DebrisStatus::Destroy)
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

    fn draw_current(&self, player: Player) {
        if player.game_status == GameStatus::GameRunning {
            plot(
                '*',
                self.col,
                self.row,
                ColorCode::new(self.color, Color::Black),
            );
        }
    }
}
