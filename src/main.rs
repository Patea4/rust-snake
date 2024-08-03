use std::{error::Error, io::{BufWriter, Stdout, Write}};
use terminal_size::terminal_size;
use rand::Rng;
use std::{thread, time};

static ESCAPE: &str = "\x1B[";

struct Vec2d {
    vec: Vec<Vec<String>>,
    width: usize,
    height: usize
}

impl Vec2d {
    fn new(width: u16, height: u16) -> Vec2d{
        Vec2d {
            vec: vec![vec![" ".to_string(); width.into()]; height.into()],
            width : width.into(),
            height : height.into(),
        }
    }

    fn pick_random(&self) -> (usize, usize) {
        let x = rand::thread_rng().gen_range(0..self.width);
        let y = rand::thread_rng().gen_range(1..self.height);
        (x, y)
    }

    fn write_grid(&self, buffer: &mut BufWriter<Stdout>) {
        for i in 1..self.height {
            for j in 1..self.width {
                write!(buffer, "{}", self.vec[i][j]).expect("Cannot write grid");
            }
            writeln!(buffer).expect("Error writing new line");
        }
    }

    fn write_food(&mut self, food: &Food) {
        self.vec[food.y][food.x] = "o".into();
    }
    
    fn delete_food(&mut self, food: &Food) {
        self.vec[food.y][food.x] = " ".into(); 
    }

    fn write_snake(&mut self, snake: &Snake) {
        let head = snake.body[0];
        self.vec[head.1][head.0] = "0".into();

        for item in snake.body.iter().skip(1) {
           self.vec[item.1][item.0] = "o".into();
        }
    }
}

struct Food {
    x: usize,
    y: usize
}

impl Food {
    fn new(grid: &Vec2d) -> Food{
        let (x, y) = grid.pick_random();
        Food {x, y}
    }
} 

struct Snake {
    direction: Direction,
    body: Vec<(usize, usize)>
}

impl Snake {
    fn new(grid: &Vec2d) -> Snake {
        let x = grid.width / 2;
        let y = grid.height / 2;
        Snake {
            direction: Direction::North,
            body: vec![(x,y),(x,y+1)]
        }
    }

    fn increase_size(&mut self) {
        let last = self.body.last().unwrap();
        let new_part = match self.direction {
            Direction::North => {
                (last.0,last.1 + 1)
            },
            Direction::East => {
                (last.0 + 1, last.1)
            },
            Direction::South => {
                (last.0, last.1 - 1)
            },
            Direction::West => {
                (last.0 - 1, last.1)
            }
        };
        self.body.push(new_part);
    }
}

enum Direction {
    North,
    East,
    South,
    West
}

fn clear(buffer: &mut BufWriter<Stdout>) {
    write!(buffer, "{ESCAPE}H").expect("Couldnt't add CLEAR to buffer");
    write!(buffer, "{ESCAPE}J").expect("Couldnt't add CLEAR to buffer");
}

fn render(grid: &mut Vec2d, buffer: &mut BufWriter<Stdout>, food: &Food, snake: &Snake) {
    clear(buffer);
    grid.write_food(food);
    grid.write_snake(snake);
    grid.write_grid(buffer);
    buffer.flush().expect("error flushing");
}

fn main() -> Result<(), Box<dyn Error>> {
    print!("{ESCAPE}?25l"); // Removes Cursor
    let size = terminal_size();
    let size = size.unwrap();
    let width: u16 = size.0.0;
    let height: u16 = size.1.0;
    
    let mut grid:  Vec2d = Vec2d::new(width, height);
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout);
     
    let mut food = Food::new(&grid);
    let mut snake = Snake::new(&grid);

    loop {
        grid.delete_food(&food);

        food = Food::new(&grid);
        snake.increase_size();

        // render(&mut grid, &mut out, &food, &snake);
        thread::sleep(time::Duration::from_secs(1));

    }
}
