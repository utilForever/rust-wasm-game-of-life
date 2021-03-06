extern crate js_sys;
extern crate fixedbitset;
extern crate web_sys;

mod utils;

use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys::console;
use fixedbitset::FixedBitSet;

// A macro to provide 'println!(..)'-style syntax for 'console.log' logging.
#[allow(unused_macros)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    #[allow(dead_code)]
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        }
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 {
            self.height - 1
        } else {
            row - 1
        };
    
        let south = if row == self.height - 1 {
            0
        } else {
            row + 1
        };
    
        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };
    
        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };
    
        let nw = self.get_index(north, west);
        count += self.cells[nw] as u8;
    
        let n = self.get_index(north, column);
        count += self.cells[n] as u8;
    
        let ne = self.get_index(north, east);
        count += self.cells[ne] as u8;
    
        let w = self.get_index(row, west);
        count += self.cells[w] as u8;
    
        let e = self.get_index(row, east);
        count += self.cells[e] as u8;
    
        let sw = self.get_index(south, west);
        count += self.cells[sw] as u8;
    
        let s = self.get_index(south, column);
        count += self.cells[s] as u8;
    
        let se = self.get_index(south, east);
        count += self.cells[se] as u8;
    
        count
    }
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");

        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                /*
                log!(
                    "cell[{}, {}] is initially {:?} and has {} live neighbors",
                    row,
                    col,
                    if cell == true { Cell::Alive } else { Cell::Dead },
                    live_neighbors
                );
                */

                next.set(idx, match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (true, x) if x < 2 => false,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (true, 2) | (true, 3) => true,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (true, x) if x > 3 => false,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (false, 3) => true,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                });

                // log!("    it becomes {:?}", if self.cells[idx] == true { Cell::Alive } else { Cell::Dead });

                /*
                if cell == true && next[idx] == false {
                    log!("cell[{}, {}] transitioned Alive to Dead", row, col);
                } else if cell == false && next[idx] == true {
                    log!("cell[{}, {}] transitioned Dead to Alive", row, col);
                }
                */
            }
        }

        self.cells = next;
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();

        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, js_sys::Math::random() < 0.5);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn reset(&mut self) {
        for i in 0..(self.width * self.height) as usize {
            self.cells.set(i, js_sys::Math::random() < 0.5);
        }
    }

    pub fn reset_all_dead(&mut self) {
        for i in 0..(self.width * self.height) as usize {
            self.cells.set(i, false);
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    /// Set the width of the universe.
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        for i in 0..(self.width * self.height) as usize { self.cells.set(i, false) }
    }

    /// Set the height of the universe.
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        for i in 0..(self.width * self.height) as usize { self.cells.set(i, false) }
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, match self.cells[idx] {
            true => false,
            false => true,
        });
    }

    pub fn insert_glider(&mut self, row: u32, column: u32) {
        if row < 1 || column < 1 || row > self.height - 2 || column > self.width - 2 {
            return
        }

        let pattern = [
            false, true, false,
            false, false, true,
            true, true, true
        ];
        let mut pattern_idx = 0;

        for i in (row-1)..(row+2) {
            for j in (column-1)..(column+2) {
                let idx = self.get_index(i, j);
                self.cells.set(idx, pattern[pattern_idx]);
                pattern_idx += 1;
            }
        }
    }

    pub fn insert_pulsar(&mut self, row: u32, column: u32) {
        if row < 7 || column < 7 || row > self.height - 8 || column > self.width - 8 {
            return
        }

        let pattern = [
            false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, true, true, true, false, false, false, true, true, true, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
            false, true, false, false, false, false, true, false, true, false, false, false, false, true, false,
            false, true, false, false, false, false, true, false, true, false, false, false, false, true, false,
            false, true, false, false, false, false, true, false, true, false, false, false, false, true, false,
            false, false, false, true, true, true, false, false, false, true, true, true, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, true, true, true, false, false, false, true, true, true, false, false, false,
            false, true, false, false, false, false, true, false, true, false, false, false, false, true, false,
            false, true, false, false, false, false, true, false, true, false, false, false, false, true, false,
            false, true, false, false, false, false, true, false, true, false, false, false, false, true, false,
            false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, true, true, true, false, false, false, true, true, true, false, false, false,
            false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
        ];
        let mut pattern_idx = 0;

        for i in (row-7)..(row+8) {
            for j in (column-7)..(column+8) {
                let idx = self.get_index(i, j);
                self.cells.set(idx, pattern[pattern_idx]);
                pattern_idx += 1;
            }
        }
    }
}

impl Universe {
    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[u32] {
        &self.cells.as_slice()
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == 0 { '???' } else { '???' };
                write!(f, "{}", symbol)?;
            }

            write!(f, "\n")?;
        }

        Ok(())
    }
}