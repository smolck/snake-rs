use crate::shader;

use rand::prelude::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Direction {
    Stationary = 0,
    Right,
    Left,
    Up,
    Down,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Tile {
    Empty = 0,
    Food,
    SnakeBody,
    SnakeHead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    x: usize,
    y: usize,
}

struct Snake {
    tiles: Vec<Pos>,
    direction: Direction,
}

pub struct Game {
    board: Vec<Tile>,
    cols: usize,
    rows: usize,

    board_width: f32,
    board_height: f32,

    board_size: usize,

    // each tile is a square
    tile_size: f32,

    snake: Snake,

    food_location: usize,
}

fn move_in_direction(tile: Pos, direction: Direction) -> Pos {
    use Direction::*;

    // println!("move in direction log {:?}", tile);
    match direction {
        Stationary => tile,
        Right => Pos {
            x: tile.x + 1,
            y: tile.y,
        },
        Left => Pos {
            x: tile.x - 1,
            y: tile.y,
        },
        Up => Pos {
            x: tile.x,
            y: tile.y + 1,
        },
        Down => Pos {
            x: tile.x,
            y: tile.y - 1,
        },
    }
}

impl Game {
    pub fn new(board_width: f32, board_height: f32, tile_size: f32) -> Self {
        let cols = (board_width / tile_size) as usize;
        let rows = (board_height / tile_size) as usize;

        let board_size = rows * cols;

        // Make starting tile the center tile
        let starting_col = cols / 2;
        let starting_row = rows / 2;
        let starting_tile = starting_row * cols + starting_col;

        // println!("{}, {}", starting_col, starting_row);
        let snake = Snake {
            tiles: vec![Pos {
                x: starting_col,
                y: starting_row,
            }],
            direction: Direction::Stationary,
        };

        let mut board: Vec<Tile> = std::iter::repeat(Tile::Empty).take(board_size).collect();
        board[starting_tile] = Tile::SnakeHead;

        let food_location = Self::generate_new_food(&board);
        board[food_location] = Tile::Food;

        Self {
            board_size,
            board_width,
            board_height,
            board,

            tile_size,

            rows,
            cols,

            snake,
            food_location,
        }
    }

    pub fn reset(&mut self) {
        *self = Game::new(self.board_width, self.board_height, self.tile_size);
    }

    fn generate_new_food(board: &Vec<Tile>) -> usize {
        board
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                if matches!(e, Tile::Empty) {
                    Some(i)
                } else {
                    None
                }
            })
            .choose(&mut rand::thread_rng())
            .unwrap()
    }

    pub fn change_direction(&mut self, new_direction: Direction) {
        // If length is 1, we can move in any direction and also the next check
        // is gonna have an out of bounds array access, so change direction and return
        if self.snake.tiles.len() == 1 {
            self.snake.direction = new_direction;
            return;
        }

        // Can't move into itself
        match new_direction {
            Direction::Right => {
                if self.snake.tiles[0].x + 1 == self.snake.tiles[1].x {
                    return;
                }
            }
            Direction::Left => {
                if self.snake.tiles[0].x - 1 == self.snake.tiles[1].x {
                    return;
                }
            }
            Direction::Up => {
                if self.snake.tiles[0].y + 1 == self.snake.tiles[1].y {
                    return;
                }
            }
            Direction::Down => {
                if self.snake.tiles[0].y - 1 == self.snake.tiles[1].y {
                    return;
                }
            }
            Direction::Stationary => unreachable!(),
        }
        self.snake.direction = new_direction;
    }

    pub fn current_direction(&self) -> Direction {
        self.snake.direction
    }

    /// Updates game. Must be called manually after every change
    /// if it returns false then game over
    pub fn update(&mut self) -> bool {
        if matches!(self.snake.direction, Direction::Stationary) {
            return true;
        }

        // Collision detection with wall
        match self.snake.direction {
            Direction::Right => {
                if self.snake.tiles[0].x == self.cols - 1 {
                    return false;
                }
            }
            Direction::Left => {
                if self.snake.tiles[0].x == 0 {
                    return false;
                }
            }
            Direction::Up => {
                if self.snake.tiles[0].y == self.rows - 1 {
                    return false;
                }
            }
            Direction::Down => {
                if self.snake.tiles[0].y == 0 {
                    return false;
                }
            }
            _ => {}
        }

        // Collision with self
        let head_tile = self.snake.tiles[0];
        for tile in self.snake.tiles[1..].iter() {
            if head_tile.x == tile.x && head_tile.y == tile.y {
                return false;
            }
        }

        // Move snake in direction
        if !matches!(self.snake.direction, Direction::Stationary) {
            if self.snake.tiles[0].x == 0 && matches!(self.snake.direction, Direction::Left) {
                // gotta do this early to avoid overflow cuz usize lol
                return false;
            }

            let new_snake_head = move_in_direction(self.snake.tiles[0], self.snake.direction);
            if new_snake_head.x > self.cols || new_snake_head.y > self.rows {
                // Game over we hit the edge of the board
                return false;
            }

            // Move snake and update board
            let last_snake = self.snake.tiles.last().unwrap();
            let head = self.snake.tiles[0];
            self.board[last_snake.y * self.cols + last_snake.x] = Tile::Empty;
            if *last_snake != head {
                self.board[head.y * self.cols + head.x] = Tile::SnakeBody;
            }

            let mut i = self.snake.tiles.len() - 1;
            while i > 0 {
                self.snake.tiles[i] = self.snake.tiles[i - 1];

                i -= 1;
            }
            self.snake.tiles[0] = new_snake_head;
            self.board[new_snake_head.y * self.cols + new_snake_head.x] = Tile::SnakeHead;
        }

        // If eat food handle that shit
        if (self.snake.tiles[0].y * self.cols + self.snake.tiles[0].x) == self.food_location {
            self.board[self.food_location] = Tile::Empty;
            self.food_location = Self::generate_new_food(&self.board);
            self.board[self.food_location] = Tile::Food;

            let last_tile = self.snake.tiles.last().unwrap();
            // TODO(smolck)
            let new_tile = if last_tile.x == self.cols {
                Pos {
                    x: last_tile.x,
                    y: last_tile.y + 1,
                }
            } else {
                Pos {
                    x: last_tile.x + 1,
                    y: last_tile.y,
                }
            };
            self.snake.tiles.push(new_tile);
            self.board[new_tile.y * self.cols + new_tile.x] = Tile::SnakeBody;
        }

        true
    }

    pub fn render_data(&self) -> Vec<shader::Vertex> {
        let mut col = 0;
        let mut row = 0;

        let mut x = 0.;
        let mut y = 0.;
        let tile_size = self.tile_size;

        let mut vertices = Vec::with_capacity(self.board.len() * 6); // * 6 cuz 6 vertices per
                                                                     // square, TODO could decrease with
                                                                     // instancing?
        for tile in &self.board {
            use shader::Vertex;
            /*if matches!(tile, Tile::SnakeHead) {
                println!("render_data log x: {}, y: {}, {:?}", x, y, tile);
            }*/

            let ps = shader::square_for_pos(self.board_width, self.board_height, x, y, tile_size);
            let coloridx = match tile {
                // See main.rs COLORS decl for explanation
                Tile::Empty => 1,
                Tile::SnakeBody | Tile::SnakeHead => 0,
                Tile::Food => 2,
            };
            vertices.push(Vertex {
                position: [ps[0], ps[1]],
                coloridx,
            });
            vertices.push(Vertex {
                position: [ps[2], ps[3]],
                coloridx,
            });
            vertices.push(Vertex {
                position: [ps[4], ps[5]],
                coloridx,
            });
            vertices.push(Vertex {
                position: [ps[6], ps[7]],
                coloridx,
            });
            vertices.push(Vertex {
                position: [ps[8], ps[9]],
                coloridx,
            });
            vertices.push(Vertex {
                position: [ps[10], ps[11]],
                coloridx,
            });

            x += tile_size;
            col += 1;

            if col == self.cols {
                col = 0;
                row += 1;

                x = 0.;
                y += tile_size;
            }
        }

        vertices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn util_debug_board(game: &Game) {
        for i in 0..game.rows {
            for j in 0..game.cols {
                let text = match game.board[i * game.cols + j] {
                    Tile::Empty => " ",
                    Tile::Food => "F",
                    Tile::SnakeBody => "b",
                    Tile::SnakeHead => "S",
                };
                print!("[{text}]");
            }
            println!();
        }
        println!();
    }

    fn util_idx_for_pos(pos: Pos, cols: usize) -> usize {
        pos.y * cols + pos.x
    }

    #[test]
    fn board_initializes_properly() {
        let game = super::Game::new(100., 100., 10.);

        assert_eq!(game.board.len(), (100 / 10) * (100 / 10));
        assert_eq!(game.snake.tiles.len(), 1);
        assert_eq!(game.snake.tiles[0].x, 5);
        assert_eq!(game.snake.tiles[0].y, 5);
    }

    #[test]
    fn snake_eats_food_properly_when_small_len() {
        let mut game = super::Game::new(100., 100., 10.);

        // Setup snake about to eat food
        let food_x = game.food_location % game.cols;
        let food_y = game.food_location / game.cols;
        game.snake.tiles[0] = Pos {
            x: food_x - 1,
            y: food_y,
        };
        game.change_direction(Direction::Right);

        let food_location = game.food_location;
        assert!(game.update());

        // New food is moved to a new location
        assert_ne!(food_location, game.food_location);
        // Snake is increased by one
        assert_eq!(game.snake.tiles.len(), 2);
        // Snake head is where food is
        assert_eq!(
            util_idx_for_pos(game.snake.tiles[0], game.cols),
            food_location
        );
        // New end of snake is behind snake head
        assert_eq!(game.snake.tiles[1].x, food_location % game.cols + 1);
    }
}
