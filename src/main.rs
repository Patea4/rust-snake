use std::{borrow::Borrow, error::Error, io::{BufWriter, Stdout, Write}};
use crossterm::{event::{poll, read, Event, KeyCode, KeyEvent}, terminal::{disable_raw_mode, enable_raw_mode}};
use terminal_size::terminal_size;
use rand::Rng;
use std::fmt;
use std::time::{Duration, Instant};
use std::{thread, time};

const MAX_INTERVAL: u16 = 300;
const MIN_INTERVAL: u16 = 50;
const MAX_SPEED: u16 = 250;

static ESCAPE: &str = "\x1B[";

struct Vec2d {
    vec: Vec<Vec<String>>,
    width: usize,
    height: usize
}

#[derive(Debug)]
enum Command {
    Quit,
    Turn(Direction)
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

    fn write_grid(&self, buffer: &mut BufWriter<Stdout>) -> Result<(), Box<dyn std::error::Error>>{
        let total_height = self.height * 2;
        let total_width = self.width * 3;
        let height_difference = total_height - self.height;
        let width_difference = total_width - self.width;

        // Writing top padding
        for _i in 1..height_difference/2 {
            write!(buffer, "\r\n")?;
        }
        
        // writing the top border and offsetting it to match the play area
        for _j in 1..width_difference/4 + 3{
            write!(buffer, " ")?;
        }
        write!(buffer, "+")?;
        for _i in 1..self.width * 2 - 2{
            write!(buffer, "-")?;
        }
        write!(buffer, "+")?;
        write!(buffer, "\r\n")?;

        for i in 1..self.height {
            // left padding
            for j in 1..width_difference/4 + 3{
                write!(buffer, " ")?;
            }

            write!(buffer, "|")?;
            for j in 1..self.width {
                if j != self.width - 1 {
                    write!(buffer, "{} ", self.vec[i][j])?;
                } else {
                    write!(buffer, "{}", self.vec[i][j])?;
                }
            }
            write!(buffer, "|")?;
            write!(buffer,"\r\n")?;
        }

        // writing the border and offsetting it to match the play area
        for _j in 1..width_difference/4 + 3{
            write!(buffer, " ")?;
        }
        write!(buffer, "+")?;
        for _i in 1..self.width * 2 - 2{
            write!(buffer, "-")?;
        }
        write!(buffer, "+")?;
        write!(buffer, "\r\n")?;

        Ok(())
    }

    fn write_food(&mut self, food: &Food) {
        self.vec[food.y][food.x] = "O".into();
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
        
        if let Some(item) = snake.last {self.vec[item.1][item.0] = " ".into()}
    }

}

struct Food {
    x: usize,
    y: usize
}

impl Food {
    fn new(grid: &Vec2d, old_food: Option<Food>) -> Food{
        let (mut x, mut y) = grid.pick_random();
        if let Some(old) = old_food {
            if x == old.x || y == old.y {
                (x, y) = grid.pick_random();
            }
        }
        Food {x, y}
    }
} 

struct Snake {
    direction: Direction,
    body: Vec<(usize, usize)>,
    last: Option<(usize, usize)>
}

#[derive(Debug, Clone)]
struct OutOfBoundsError;

impl fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Snake out of bounds")
    }
}

impl Snake {
    fn new(grid: &Vec2d) -> Snake {
        let x = grid.width / 2;
        let y = grid.height / 2;
        Snake {
            direction: Direction::North,
            body: vec![(x,y),(x,y+1)],
            last: None
        }
    }

    fn increase_size(&mut self) {
        let first = self.body.first().unwrap();
        let new_part = match self.direction {
            Direction::North => {
                (first.0,first.1 - 1)
            },
            Direction::East => {
                (first.0 + 1, first.1)
            },
            Direction::South => {
                (first.0, first.1 + 1)
            },
            Direction::West => {
                (first.0 - 1, first.1)
            }
        };
        self.body.insert(0,new_part);
    }

    fn update_position(&mut self) -> Result<(), OutOfBoundsError>{
        // add new element at start of body vector, making it the new head at the current
        // direction. Then remove the last element of the body vector
        let head = self.body.first().unwrap();
        let new_position = match self.direction {
            Direction::North => {
                if head.1 == 0 {
                    return Err(OutOfBoundsError)
                }
                (head.0, head.1 - 1)
            },
            Direction::East => {
                (head.0 + 1, head.1)
            },
            Direction::South => {
                (head.0, head.1 + 1)
            },
            Direction::West => {
                if head.0 == 0 {
                    return Err(OutOfBoundsError)
                }
                (head.0 - 1, head.1)
            }
        };
        self.body.insert(0, new_position);
        self.last = self.body.last().copied();
        self.body.pop();
        Ok(())
    }
}

#[derive(PartialEq, Clone, Debug)]
enum Direction {
    North,
    East,
    South,
    West
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East
        }
    }
}

fn clear(buffer: &mut BufWriter<Stdout>) {
    write!(buffer, "{ESCAPE}H").expect("Couldnt't add CLEAR to buffer");
    write!(buffer, "{ESCAPE}J").expect("Couldnt't add CLEAR to buffer");
}

fn write_score(buffer: &mut BufWriter<Stdout>, score: u16, grid: &Vec2d) {
    let width_difference = grid.width.borrow();
    for _i in 1..(width_difference / 4) + 11{
        write!(buffer, " ").expect("Cannot write left padding for score");
    }
    write!(buffer, "Score: {}", score).expect("Cannot write score");
}

fn render(grid: &mut Vec2d, buffer: &mut BufWriter<Stdout>, food: &Food, snake: &Snake, score: u16) {
    clear(buffer);
    grid.write_food(food);
    grid.write_snake(snake);
    grid.write_grid(buffer);
    write_score(buffer, score, grid);
    buffer.flush().expect("error flushing");
}


fn get_input(wait_for: Duration) -> Option<Command> {
    let key_event = wait_for_key_event(wait_for)?;

    match key_event.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Command::Quit),
        KeyCode::Char('w') => Some(Command::Turn(Direction::North)),
        KeyCode::Char('d') => Some(Command::Turn(Direction::East)),
        KeyCode::Char('s') => Some(Command::Turn(Direction::South)),
        KeyCode::Char('a') => Some(Command::Turn(Direction::West)),
        _ => None
    }
}

fn wait_for_key_event(wait_for: Duration) -> Option<KeyEvent> {
    if poll(wait_for).ok()? {
        let event = read().ok()?;
        if let Event::Key(key_event) = event {
            return Some(key_event);
        }
    }
    None
}

fn calculate_interval(speed: u16) -> Duration {
    let speed = MAX_SPEED - speed;
    Duration::from_millis(
        (MIN_INTERVAL + (((MAX_INTERVAL - MIN_INTERVAL) / MAX_SPEED) * speed)) as u64
    )
}

enum Collision {
    Food,
    Obstacle,
}

fn collision_with_food(snake: &mut Snake) {
    snake.increase_size();
}

fn check_collison(snake: &mut Snake, grid: &Vec2d) -> Option<Collision> {
    let head_position = snake.body.first().unwrap();
    if head_position.1 == grid.vec.len() || head_position.0 == grid.vec[0].len(){
        Some(Collision::Obstacle)
    } else if grid.vec[head_position.1][head_position.0] == "O" {
        collision_with_food(snake);
        Some(Collision::Food)
    } else if grid.vec[head_position.1][head_position.0] == "o" {
        Some(Collision::Obstacle)
    } else {
        return None
    }
}

// fn write_end_screen(buffer: &mut BufWriter<Stdout>, score: u16, height: u16, width: u16) -> Result<(), Box<dyn std::error::Error>> {
//     let height = height / 2;
//     let width = width / 3;
//     for _ in 1..height - 1{
//         write!(buffer, "\r\n").expect("a");
//     }
//     for _ in 1..width {
//         write!(buffer, " ").expect("b");
//     }
//     let score_line = format!("| Score: {score}");
//     write!(buffer, "{score_line}").expect("Error writing score");
//     let difference: u16 = width - (score_line.len() as u16);
// 
//     for _ in 1..10 {
//         write!(buffer, " ").expect("c");
//     }
//     write!(buffer,"|\r\n").expect("d");
//     
//     Ok(())
// }

fn main() -> Result<(), Box<dyn Error>> {
    let speed = 150;
    let mut score = 0;
    print!("{ESCAPE}?25l"); // Removes Cursor
    let size = terminal_size();
    let size = size.unwrap();
    let width: u16 = size.0.0 / 3;
    let height: u16 = size.1.0 / 2;

    let mut grid:  Vec2d = Vec2d::new(width, height);
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout);
     
    let _ = enable_raw_mode();
    let mut food = Food::new(&grid, None);
    let mut snake = Snake::new(&grid);
    
    let mut done = false;
    while !done {

        let interval = calculate_interval(speed);
        let now = Instant::now();
        let direction = snake.direction.clone();

        while now.elapsed() < interval {
            if let Some(command) = get_input(interval - now.elapsed()) {
                match command {
                    Command::Quit => {
                        done = true;
                        break;
                    }
                    Command::Turn(towards) => {
                        if direction != towards && direction.opposite() != towards {
                            snake.direction = towards;
                        }
                    }
                }
            }
        }
        
        if let Err(OutOfBoundsError) = snake.update_position() {
            println!("end of times");
            done = true;
        }
        match check_collison(&mut snake, &grid) {
            Some(Collision::Food) => {
                grid.delete_food(&food);
                food = Food::new(&grid, Some(food));
                score += 1;
            },
            Some(Collision::Obstacle) => {
                break;
            },
            _ => ()
        }
        render(&mut grid, &mut out, &food, &snake, score);

    }
    
    out.flush().expect("Error flushing");
    //thread::sleep(time::Duration::from_millis(500));

    let _ = disable_raw_mode();
    println!("{ESCAPE}?25h");
    println!("Game over! Score: {}", score);
    Ok(())
}

