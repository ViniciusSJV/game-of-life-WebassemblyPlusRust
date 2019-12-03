mod utils;

use wasm_bindgen::prelude::*;

extern crate js_sys;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use web_sys::console;

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

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
	
	pub fn new() -> Universe {
		//utils::set_panic_hook();
		
        let width = 128;
        let height = 128;

        let cells = (0..width * height)
            .map(|_i| {
                if js_sys::Math::random() < 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }
	
	pub fn restart(&mut self) {
		let cells = (0..self.width * self.height)
            .map(|_i| {
                if js_sys::Math::random() < 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();
			
		self.cells = cells;	
	}

    pub fn render(&self) -> String {
        self.to_string()
    }
	
	pub fn tick(&mut self, range: u32) {
		let _timer = Timer::new("Universe::tick");

		let mut next = {
			let _timer = Timer::new("allocate next cells");
			self.cells.clone()
		};

		{
			let _timer = Timer::new("new generation");
			for row in range..self.height {
				for col in range..self.width {
					let idx = self.get_index(row, col);
					let cell = self.cells[idx];
					let live_neighbors = self.live_neighbor_count(row, col);

					let next_cell = match (cell, live_neighbors) {
						// Rule 1: Any live cell with fewer than two live neighbours
						// dies, as if caused by underpopulation.
						(Cell::Alive, x) if x < 2 => Cell::Dead,
						// Rule 2: Any live cell with two or three live neighbours
						// lives on to the next generation.
						(Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
						// Rule 3: Any live cell with more than three live
						// neighbours dies, as if by overpopulation.
						(Cell::Alive, x) if x > 3 => Cell::Dead,
						// Rule 4: Any dead cell with exactly three live neighbours
						// becomes a live cell, as if by reproduction.
						(Cell::Dead, 3) => Cell::Alive,
						// All other cells remain in the same state.
						(otherwise, _) => otherwise,
					};

					next[idx] = next_cell;
				}
			}
		}

		let _timer = Timer::new("free old cells");
		self.cells = next;
	}
	
	fn get_index(&self, row: u32, column: u32) -> usize {
        let (_row_normalize, _column_normalize) = self.normalize_coordinate(row, column);
		(_row_normalize * self.width + _column_normalize) as usize
    }
	
	fn normalize_coordinate(&self, mut row: u32, mut col: u32) -> (u32, u32) {
		
		if row < 0 {
			row = self.height - 1;
		} else if row > self.height - 1{
			row = 0;
		};
		
		if col < 0 {
			col = self.width - 1;
		} else if col > self.width - 1 {
			col = 0;
		};
		
		(row, col)
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
	
	pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
	
	pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }
	
	pub fn toggle_live_cell(&mut self) {
		for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
				let mut cell = self.cells[idx];
				
				if cell == Cell::Dead {
					cell = Cell::Alive;
				} else {
					cell = Cell::Dead;
				}	
				self.cells[idx] = cell 	
			}
		}	
	}
	
	pub fn create_glider(&mut self, row: u32, col: u32) {
		let idx1 = self.get_index(row, col);
		let idx2 = self.get_index(row, col + 1);
		let idx3 = self.get_index(row, col + 2);
		
		self.cells[idx1] = Cell::Alive; 
		self.cells[idx2] = Cell::Alive; 
		self.cells[idx3] = Cell::Alive; 
	}
	
	pub fn create_pulsar_gerator(&mut self, row: u32, col: u32) {
		let idx_center_pulsar = self.get_index(row, col);
		self.cells[idx_center_pulsar] = Cell::Dead; 
		
		let mut index = self.get_index(row - 2, col);
		self.cells[index] = Cell::Alive; 
		
		index = self.get_index(row + 2, col);
		self.cells[index] = Cell::Alive;
		
		index = self.get_index(row + 1, col);
		self.cells[index] = Cell::Alive; 
		
		index = self.get_index(row - 1, col);
		self.cells[index] = Cell::Alive; 
		
		index = self.get_index(row, col + 1);
		self.cells[index] = Cell::Alive;
		
		index = self.get_index(row, col - 1);
		self.cells[index] = Cell::Alive; 
		
		index = self.get_index(row + 1, col + 1);
		self.cells[index] = Cell::Alive; 
		
		index = self.get_index(row + 1, col - 1);
		self.cells[index] = Cell::Alive; 
		
		index = self.get_index(row - 1, col + 1);
		self.cells[index] = Cell::Alive; 
		
		index = self.get_index(row - 1, col - 1);
		self.cells[index] = Cell::Alive; 
	}
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}