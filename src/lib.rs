#![no_std]

use num::Integer;
use heapless::Vec;
use oorandom::{self, Rand32};
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, plot_str, plot_num, clear_row
};

use core::{
    clone::Clone, cmp::{Eq, PartialEq}, marker::Copy, prelude::rust_2024::derive
};

const DEBRIS_COLORS: [Color; 13] = [Color::Blue, Color::Green, Color::Cyan, Color::Red, Color::Magenta,
                                    Color::LightGray, Color::LightBlue, Color::LightGreen, Color::LightCyan,
                                    Color::LightRed, Color::Pink, Color::Yellow, Color::White];
const CW_SPAWN_RATE: u32 = 8;
const RMT_SPAWN_RATE: u32 = 6;
const N_SPAWN_RATE: u32 = 3;
const CW_LOWER_SPEED: u32 = 3;
const CW_UPPER_SPEED: u32 = 7;
const RMT_LOWER_SPEED: u32 = 2;
const RMT_UPPER_SPEED: u32 = 5;
const N_LOWER_SPEED: u32 = 1;
const N_UPPER_SPEED: u32 = 2;

#[derive(Copy, Clone, Eq, PartialEq)]
enum GameStatus {
    GameRunning,
    GameStopped
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Difficulty {
    Undefined,
    Cakewalk,
    RMT,
    Nightmare
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum DebrisStatus {
    Normal,
    ScorePoint,
    Destroy
}

#[derive(Clone, Eq, PartialEq)]
pub struct SpaceDebrisGame {
    player: Player,
    debris: Vec<Debris, 50>,
    score: u32,
    cw_high_score: u32,
    rmt_high_score: u32,
    n_high_score: u32,
    spawn_countdown: u32,
    spawn_rate: u32,
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

fn safe_add<const LIMIT: usize>(a: usize, b: usize) -> usize {
    (a + b).mod_floor(&LIMIT)
}

fn add1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, 1)
}

fn sub1<const LIMIT: usize>(value: usize) -> usize {
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
            spawn_countdown: 0,
            spawn_rate: 0,
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
            game_status: GameStatus::GameStopped
        }
    }
}

impl Debris {
    fn new(num: u32, lower_speed: u32, upper_speed: u32) -> Self {
        let mut rng: Rand32 = Rand32::new(num.into());
        Self {
            col: BUFFER_WIDTH - 1,
            row: rng.rand_range(2..BUFFER_HEIGHT as u32) as usize,
            dx: rng.rand_range(lower_speed..upper_speed) as usize,
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
                GameStatus::GameStopped => {
                    self.player.dy = 0;
                    self.update_high_score();
                    self.display_title_screen();
                }
            }
        }
        let mut deleted_debris: Vec<usize, 50> = Vec::<usize, 50>::new();
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

    pub fn display_title_screen(&self) {
        let color_white: ColorCode = ColorCode::new(Color::White, Color::Black);
        let color_red: ColorCode = ColorCode::new(Color::LightRed, Color::Black);
        let title_text: &str = "SPACE JUNK";
        let cw_score_text: &str = "High Score (Cakewalk): ";
        let rmt_score_text: &str = "High Score (Road Most Travelled): ";
        let n_score_text: &str = "High Score (Nightmare): ";
        let control_text: &str = "Controls: Up/Down Arrow Keys";
        let difficulty_text: &str = "Press 1 to Play Cakewalk, 2 for Road Most Travelled, 3 for Nightmare";
        plot_str(title_text, BUFFER_WIDTH / 2 - 5, BUFFER_HEIGHT / 2 - 4, color_red);
        plot_str(cw_score_text, BUFFER_WIDTH / 2 - 12, BUFFER_HEIGHT / 2 - 2, color_white);
        plot_num(self.cw_high_score as isize, cw_score_text.len() + 28, BUFFER_HEIGHT / 2 - 2, color_white);
        plot_str(rmt_score_text, BUFFER_WIDTH / 2 - 18, BUFFER_HEIGHT / 2 - 1, color_white);
        plot_num(self.rmt_high_score as isize, rmt_score_text.len() + 22, BUFFER_HEIGHT / 2 - 1, color_white);
        plot_str(n_score_text, BUFFER_WIDTH / 2 - 12, BUFFER_HEIGHT / 2, color_white);
        plot_num(self.n_high_score as isize, n_score_text.len() + 28, BUFFER_HEIGHT / 2, color_white);
        plot_str(control_text, BUFFER_WIDTH / 2 - 13, BUFFER_HEIGHT / 2 + 2, color_white);
        plot_str(difficulty_text, BUFFER_WIDTH / 2 - 34, BUFFER_HEIGHT / 2 + 3, color_white);
    }

    pub fn key(&mut self, key: DecodedKey) {
        if self.player.game_status == GameStatus::GameStopped {
            if key == DecodedKey::Unicode('1') {
                self.difficulty = Difficulty::Cakewalk;
                self.reset();
            } else if key == DecodedKey::Unicode('2') {
                self.difficulty = Difficulty::RMT;
                self.reset();
            } else if key == DecodedKey::Unicode('3') {
                self.difficulty = Difficulty::Nightmare;
                self.reset();
            }
        }
        self.player.key(key);
    }

    fn increment_score(&mut self) {
        if self.player.game_status == GameStatus::GameRunning {
            self.score += 1;
            self.display_score();
        }
    }

    fn update_high_score(&mut self) {
        match self.difficulty {
            Difficulty::Undefined => {},
            Difficulty::Cakewalk => {
                if self.score > self.cw_high_score {
                    self.cw_high_score = self.score;
                }
            },
            Difficulty::RMT => {
                if self.score > self.rmt_high_score {
                    self.rmt_high_score = self.score;
                }
            },
            Difficulty::Nightmare => {
                if self.score > self.n_high_score {
                    self.n_high_score = self.score;
                }
            }
        }
    }

    fn display_score(&self) {
        let header_color: ColorCode = ColorCode::new(Color::White, Color::Black);
        let mut score_text: &str = "";
        match self.difficulty {
            Difficulty::Undefined => {},
            Difficulty::Cakewalk => score_text = "Score (Cakewalk): ",
            Difficulty::RMT => score_text = "Score (Road Most Travelled): ",
            Difficulty::Nightmare => score_text = "Score (Nightmare): "
        }
        clear_row(0, Color::Black);
        plot_str(score_text, 0, 0, header_color);
        plot_num(self.score as isize, score_text.len(), 0, header_color);
    }

    fn create_debris(&mut self) {
        self.seed_count += 1;
        if self.spawn_countdown == 0 {
            let mut lower_speed: u32 = 1;
            let mut upper_speed: u32 = 5;
            match self.difficulty {
                Difficulty::Undefined => {},
                Difficulty::Cakewalk => {
                    lower_speed = CW_LOWER_SPEED;
                    upper_speed = CW_UPPER_SPEED;
                },
                Difficulty::RMT => {
                    lower_speed = RMT_LOWER_SPEED;
                    upper_speed = RMT_UPPER_SPEED;
                },
                Difficulty::Nightmare => {
                    lower_speed = N_LOWER_SPEED;
                    upper_speed = N_UPPER_SPEED;
                }
            }
            let _ = self.debris.push(Debris::new(self.seed_count, lower_speed, upper_speed));
            self.spawn_countdown = self.spawn_rate;
        } else {
            self.spawn_countdown -= 1;
        }
    }

    fn reset(&mut self) {
        for i in 8..=15 {
            clear_row(i, Color::Black);
        }
        self.player.game_status = GameStatus::GameRunning;
        match self.difficulty {
            Difficulty::Undefined => {},
            Difficulty::Cakewalk => self.spawn_rate = CW_SPAWN_RATE,
            Difficulty::RMT => self.spawn_rate = RMT_SPAWN_RATE,
            Difficulty::Nightmare => self.spawn_rate = N_SPAWN_RATE
        }
        self.player.clear_current();
        let mut deleted_debris: Vec<usize, 50> = Vec::<usize, 50>::new();
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
        self.display_score();
    }
}

impl Player {
    fn tick(&mut self) -> Option<GameStatus> {
        self.clear_current();
        self.update_location();
        self.draw_current();
        if self.game_status == GameStatus::GameStopped {
            return Some(GameStatus::GameStopped);
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
        clear_row(0, Color::Black);
        self.game_status = GameStatus::GameStopped;
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
