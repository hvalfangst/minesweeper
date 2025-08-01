use leptos::*;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellState {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub is_mine: bool,
    pub state: CellState,
    pub adjacent_mines: u8,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            is_mine: false,
            state: CellState::Hidden,
            adjacent_mines: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameStatus {
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub board: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
    pub mines: usize,
    pub status: GameStatus,
    pub flags_remaining: i32,
    pub first_click: bool,
}

impl GameState {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let board = vec![vec![Cell::new(); width]; height];
        
        Self {
            board,
            width,
            height,
            mines,
            status: GameStatus::Playing,
            flags_remaining: mines as i32,
            first_click: true,
        }
    }

    pub fn place_mines(&mut self, avoid_row: usize, avoid_col: usize) {
        let mut rng = SmallRng::from_entropy();
        let mut mines_placed = 0;
        let mut mine_positions = HashSet::new();

        while mines_placed < self.mines {
            let row = rng.gen_range(0..self.height);
            let col = rng.gen_range(0..self.width);

            if (row, col) == (avoid_row, avoid_col) || mine_positions.contains(&(row, col)) {
                continue;
            }

            mine_positions.insert((row, col));
            self.board[row][col].is_mine = true;
            mines_placed += 1;
        }

        self.calculate_adjacent_mines();
    }

    fn calculate_adjacent_mines(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                if !self.board[row][col].is_mine {
                    let mut count = 0;
                    for dr in -1..=1 {
                        for dc in -1..=1 {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            let new_row = row as i32 + dr;
                            let new_col = col as i32 + dc;
                            if new_row >= 0 && new_row < self.height as i32 &&
                               new_col >= 0 && new_col < self.width as i32 {
                                if self.board[new_row as usize][new_col as usize].is_mine {
                                    count += 1;
                                }
                            }
                        }
                    }
                    self.board[row][col].adjacent_mines = count;
                }
            }
        }
    }

    pub fn reveal_cell(&mut self, row: usize, col: usize) {
        if self.status != GameStatus::Playing ||
           self.board[row][col].state == CellState::Flagged ||
           self.board[row][col].state == CellState::Revealed {
            return;
        }

        if self.first_click {
            self.place_mines(row, col);
            self.first_click = false;
        }

        self.board[row][col].state = CellState::Revealed;

        if self.board[row][col].is_mine {
            self.status = GameStatus::Lost;
            self.reveal_all_mines();
            return;
        }

        if self.board[row][col].adjacent_mines == 0 {
            self.reveal_adjacent_cells(row, col);
        }

        self.check_win_condition();
    }

    fn reveal_adjacent_cells(&mut self, row: usize, col: usize) {
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let new_row = row as i32 + dr;
                let new_col = col as i32 + dc;
                if new_row >= 0 && new_row < self.height as i32 &&
                   new_col >= 0 && new_col < self.width as i32 {
                    let new_row = new_row as usize;
                    let new_col = new_col as usize;
                    if self.board[new_row][new_col].state == CellState::Hidden {
                        self.reveal_cell(new_row, new_col);
                    }
                }
            }
        }
    }

    fn reveal_all_mines(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                if self.board[row][col].is_mine {
                    self.board[row][col].state = CellState::Revealed;
                }
            }
        }
    }

    pub fn toggle_flag(&mut self, row: usize, col: usize) {
        if self.status != GameStatus::Playing ||
           self.board[row][col].state == CellState::Revealed {
            return;
        }

        match self.board[row][col].state {
            CellState::Hidden => {
                if self.flags_remaining > 0 {
                    self.board[row][col].state = CellState::Flagged;
                    self.flags_remaining -= 1;
                }
            },
            CellState::Flagged => {
                self.board[row][col].state = CellState::Hidden;
                self.flags_remaining += 1;
            },
            _ => {}
        }
    }

    fn check_win_condition(&mut self) {
        let mut revealed_count = 0;
        for row in 0..self.height {
            for col in 0..self.width {
                if self.board[row][col].state == CellState::Revealed {
                    revealed_count += 1;
                }
            }
        }

        if revealed_count == (self.width * self.height) - self.mines {
            self.status = GameStatus::Won;
        }
    }

    pub fn reset(&mut self) {
        self.board = vec![vec![Cell::new(); self.width]; self.height];
        self.status = GameStatus::Playing;
        self.flags_remaining = self.mines as i32;
        self.first_click = true;
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (game_state, set_game_state) = create_signal(GameState::new(16, 16, 40));

    let handle_cell_click = move |row: usize, col: usize| {
        set_game_state.update(|state| {
            state.reveal_cell(row, col);
        });
    };

    let handle_cell_right_click = move |row: usize, col: usize| {
        set_game_state.update(|state| {
            state.toggle_flag(row, col);
        });
    };

    let reset_game = move |_| {
        set_game_state.update(|state| {
            state.reset();
        });
    };

    view! {
        <div class="game-container">
            <div class="game-header">
                <h1>"Minesweeper"</h1>
                <div class="game-info">
                    <div class="mines-counter">
                        "Mines: " {move || game_state.get().flags_remaining}
                    </div>
                    <button class="reset-button" on:click=reset_game>
                        "Reset"
                    </button>
                    <div class="status">
                        {move || match game_state.get().status {
                            GameStatus::Playing => "Playing",
                            GameStatus::Won => "You Won!",
                            GameStatus::Lost => "Game Over",
                        }}
                    </div>
                </div>
            </div>
            <div class="game-board">
                {move || {
                    let state = game_state.get();
                    (0..state.height).map(|row| {
                        view! {
                            <div class="board-row">
                                {(0..state.width).map(|col| {
                                    let cell = state.board[row][col];
                                    view! {
                                        <CellComponent
                                            cell=cell
                                            on_click=move |_| handle_cell_click(row, col)
                                            on_right_click=move |_| handle_cell_right_click(row, col)
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

#[component]
fn CellComponent(
    cell: Cell,
    on_click: impl Fn(web_sys::MouseEvent) + 'static,
    on_right_click: impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    let cell_class = match cell.state {
        CellState::Hidden => "cell cell-hidden",
        CellState::Revealed => if cell.is_mine {
            "cell cell-mine"
        } else {
            "cell cell-revealed"
        },
        CellState::Flagged => "cell cell-flagged",
    };

    let cell_content = match cell.state {
        CellState::Hidden => "".to_string(),
        CellState::Flagged => "🚩".to_string(),
        CellState::Revealed => {
            if cell.is_mine {
                "💣".to_string()
            } else if cell.adjacent_mines > 0 {
                cell.adjacent_mines.to_string()
            } else {
                "".to_string()
            }
        }
    };

    let number_class = if cell.state == CellState::Revealed && cell.adjacent_mines > 0 {
        format!("number-{}", cell.adjacent_mines)
    } else {
        "".to_string()
    };

    view! {
        <button
            class=format!("{} {}", cell_class, number_class)
            on:click=on_click
            on:contextmenu=move |e| {
                e.prevent_default();
                on_right_click(e);
            }
        >
            {cell_content}
        </button>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}