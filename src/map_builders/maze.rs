use bracket_lib::prelude::console;
use bracket_lib::random::RandomNumberGenerator;

use crate::map::{Map, TileType};
use crate::map_builders::{BuilderMap, InitialMapBuilder};

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    col: i32,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    pub fn new(row: i32, col: i32) -> Self {
        Self {
            row,
            col,
            walls: [true, true, true, true],
            visited: false,
        }
    }

    fn remove_walls(&mut self, next: &mut Cell) {
        let x = self.col - next.col;
        let y = self.row - next.row;

        if x == 1 {
            self.walls[LEFT] = false;
            next.walls[RIGHT] = false;
        } else if x == -1 {
            self.walls[RIGHT] = false;
            next.walls[LEFT] = false;
        } else if y == 1 {
            self.walls[TOP] = false;
            next.walls[BOTTOM] = false;
        } else if y == -1 {
            self.walls[BOTTOM] = false;
            next.walls[TOP] = false;
        }
    }
}

struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng: &'a mut RandomNumberGenerator,
}

impl<'a> Grid<'a> {
    pub fn new(width: i32, height: i32, rng: &'a mut RandomNumberGenerator) -> Self {
        let mut grid = Self {
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0,
            rng,
        };
        for row in 0..height {
            for col in 0..width {
                grid.cells.push(Cell::new(row, col));
            }
        }

        grid
    }

    fn calculate_index(&self, row: i32, col: i32) -> i32 {
        if row < 0 || col < 0 || col > self.width - 1 || row > self.height - 1 {
            -1
        } else {
            col + (row * self.width)
        }
    }

    fn get_available_neighbors(&self) -> Vec<usize> {
        let mut neighbors: Vec<usize> = Vec::new();

        let current_row = self.cells[self.current].row;
        let current_col = self.cells[self.current].col;

        let neighbor_indices: [i32; 4] = [
            self.calculate_index(current_row - 1, current_col),
            self.calculate_index(current_row, current_col + 1),
            self.calculate_index(current_row + 1, current_col),
            self.calculate_index(current_row, current_col - 1),
        ];
        for i in neighbor_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbors.push(*i as usize);
            }
        }
        neighbors
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbors = self.get_available_neighbors();
        if !neighbors.is_empty() {
            return if neighbors.len() == 1 {
                Some(neighbors[0])
            } else {
                Some(neighbors[(self.rng.roll_dice(1, neighbors.len() as i32) - 1) as usize])
            };
        }
        None
    }

    fn generate_maze(&mut self, build_data: &mut BuilderMap) {
        let mut done = false;
        let mut i = 0;
        while !done {
            self.cells[self.current].visited = true;
            let next = self.find_next_cell();
            console::log(format!("retrieved next cell: {:?}", next));

            match next {
                Some(next) => {
                    self.cells[next].visited = true;
                    self.backtrace.push(self.current);
                    let (lower_part, higher_part) =
                        self.cells.split_at_mut(std::cmp::max(self.current, next));
                    let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                    let cell2 = &mut higher_part[0];
                    cell1.remove_walls(cell2);
                    self.current = next;
                }
                None => {
                    if !self.backtrace.is_empty() {
                        self.current = self.backtrace[0];
                        self.backtrace.remove(0);
                    } else {
                        done = true;
                    }
                }
            }

            self.copy_to_map(&mut build_data.map);
            if i % 50 == 0 {
                build_data.take_snapshot();
            }
            i += 1;
        }
    }

    fn copy_to_map(&self, map: &mut Map) {
        for x in map.tiles.iter_mut() {
            for t in x.iter_mut() {
                *t = TileType::Wall
            }
        }

        for cell in self.cells.iter() {
            let x = ((cell.col + 1) * 2) as usize;
            let y = ((cell.row + 1) * 2) as usize;

            map.tiles[x][y] = TileType::Floor;
            if !cell.walls[TOP] {
                map.tiles[x][y - 1] = TileType::Floor
            }
            if !cell.walls[RIGHT] {
                map.tiles[x + 1][y] = TileType::Floor
            }
            if !cell.walls[BOTTOM] {
                map.tiles[x][y + 1] = TileType::Floor
            }
            if !cell.walls[LEFT] {
                map.tiles[x - 1][y] = TileType::Floor
            }
        }
    }
}

pub struct MazeBuilder {
}

impl InitialMapBuilder for MazeBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl MazeBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {

        let mut grid = Grid::new(
            (build_data.map.width / 2) - 2,
            (build_data.map.height / 2) - 2,
            rng,
        );
        grid.generate_maze(build_data);
    }
}
