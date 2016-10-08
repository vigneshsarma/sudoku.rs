use std::{env, fmt};
use std::fs::File;
use std::io::Read;

type Board = [[u8; 9]; 9];

//0|1|2
//3|4|5
//6|7|8
#[derive(Copy, Clone)]
struct Axis {
    len: usize,
    i: usize,
    present: [bool; 9]
}

#[derive(Debug)]
struct Sudoku {
    len: usize,
    data: [[u8; 9]; 9],
    rows: [Axis; 9],
    columns: [Axis; 9],
    boxes: [Axis; 9],
    grid: [[[bool; 9]; 9]; 9] // caches what values are possible in what column
}

impl Axis {
    fn new(i: usize) -> Axis {
        Axis{len: 0, i: i, present: [false; 9]}
    }

    fn add(&mut self, val: u8) {
        if self.present[(val-1) as usize] == false {
            self.present[(val-1) as usize] = true;
            self.len += 1;
        } else {
            println!("{:?} {}", self, val)
        }
    }

    fn mark_possibilities(&self, possibilities: &mut [bool; 9]) {
        for (i, item) in self.present.iter().enumerate() {
            if *item {
                possibilities[i] = false;
            }
        }
    }
}

impl fmt::Debug for Axis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, there) in self.present.iter().enumerate() {
            if !there {
                write!(f, "{}, ", i).unwrap();
            }
        }
        write!(f, "Axis {{{} ->  {}, {:?}}}\n", self.i, self.len, self.present)
    }
}

impl Sudoku {
    fn new() -> Sudoku {
        Sudoku{len: 0,
               data: [[0u8; 9]; 9],
               rows: [Axis::new(0), Axis::new(1), Axis::new(2),
                      Axis::new(3), Axis::new(4), Axis::new(5),
                      Axis::new(6), Axis::new(7), Axis::new(8)],
               columns: [Axis::new(0), Axis::new(1), Axis::new(2),
                         Axis::new(3), Axis::new(4), Axis::new(5),
                         Axis::new(6), Axis::new(7), Axis::new(8)],
               boxes: [Axis::new(0), Axis::new(1), Axis::new(2),
                       Axis::new(3), Axis::new(4), Axis::new(5),
                       Axis::new(6), Axis::new(7), Axis::new(8)],
               grid: [[[true; 9]; 9]; 9]}
    }

    fn done(&self) -> bool {
        self.len == 81
    }

    fn from_str(s: String) -> Sudoku {
        let mut problem = Sudoku::new();

        let s_ = s.replace("\n", "");
        for (i, ch) in s_.chars().enumerate() {
            if ch != '0' {
                problem.set(i/9, i%9, ch.to_digit(10).unwrap() as u8);
            }
        }
        problem.update_grid();
        problem
    }

    fn set(&mut self, r: usize, c: usize, val: u8) {
        assert_eq!(self.data[r][c], 0);
        self.len += 1;
        self.data[r][c] = val;
        self.rows[r].add(val);
        self.columns[c].add(val);
        // println!("{} {} '{}'-> {}", r, c, val, (r/3+r%3)+(c/3+c%3));
        self.boxes[row_column_to_box(r, c)].add(val);
        self.grid[r][c] = [false; 9];
    }

    fn candidates_for(&self, r: usize, c: usize) -> [bool; 9] {
        if self.data[r][c] != 0 {
            println!("{}", self.data[r][c]);
            assert!(false);
        }

        let mut possibilities:[bool; 9] = [true; 9];
        self.rows[r].mark_possibilities(&mut possibilities);
        self.columns[c].mark_possibilities(&mut possibilities);
        self.boxes[row_column_to_box(r, c)].mark_possibilities(
            &mut possibilities);

        possibilities
    }

    fn update_grid(&mut self) {
        for r in 0..9 {
            for c in 0..9 {
                if self.data[r][c] == 0 {
                    self.grid[r][c] = self.candidates_for(r, c);
                }
            }
        }
    }

    fn check_row_possible<F>(&mut self, debug: &str, location_tr: F) -> usize
        where F: Fn((usize, usize)) -> (usize, usize) {
        let mut updates = 0;
        for i in 0..9 { // iter over rows
            let mut recorder:[(usize, Vec<usize>); 9] =
                [(0, vec![]), (0, vec![]), (0, vec![]),
                 (0, vec![]), (0, vec![]), (0, vec![]),
                 (0, vec![]), (0, vec![]), (0, vec![])];

            for j in 0..9 { // iter over columns
                for k in 0..9 { // iter over possible values
                    let (r, c) = location_tr((i, j));
                    if self.grid[r][c][k] {
                        let (ref mut count, ref mut locations) = recorder[k];
                        locations.push(j);
                        *count += 1;
                    }
                }
            }
            for k in 0..9 {
                let (ref count, ref locations) = recorder[k];
                if *count == 1 {
                    let (r, c) = location_tr((i, locations[0]));
                    println!("{} {:?} -> {}", debug, (r, c), k+1);
                    self.set(r, c, (k+1) as u8);
                    updates += 1;
                }
            }
        }
        updates
    }

    fn loop_over(&mut self, debug: bool) -> usize {
        let mut updates = 0;
        for i in 0..9 {
            for j in 0..9 {
                let presense = self.grid[i][j];
                let possible = presense_array_to_vec(&presense);
                if possible.len() == 1 {
                    println!("{:?} -> {:?}", (i, j), possible);
                    self.set(i, j, possible[0]);
                    updates += 1;
                } else if !possible.is_empty() && debug {
                    println!("{:?} -> {:?}", (i, j), possible);
                }
            }
        }
        if updates == 0 {
            // let id = |(r, c)| (r, c);
            self.update_grid();
            updates = self.check_row_possible("Row", |(r, c)| (r, c));
        }
        if updates == 0 {
            // let id = |(r, c)| (r, c);
            self.update_grid();
            updates = self.check_row_possible("Column", |(r, c)| (c, r));
        }

        updates
    }

    fn solve(&mut self) {
        while !self.done() {
            if self.loop_over(false) == 0 {
                println!("---------- Failed: Cant find any more ----------. {}", self.len);
                self.update_grid();
                self.loop_over(true);
                display_board(&self.data);
                println!("---------- Failed: Cant find any more ----------. {}", self.len);
                break;
            } else {
                self.update_grid();
                println!("--------------------");
            }
        }
        if self.done() {

            println!("------- Solved -------------");
            display_board(&self.data);
            println!("------- Solved -------------");
        }
    }

}

fn presense_array_to_vec(data: &[bool; 9]) -> Vec<u8> {
    let mut options = vec!();
    for (i, item) in data.iter().enumerate() {
        if *item {
            options.push((i+1) as u8);
        }
    }
    options
}

fn row_column_to_box(r: usize, c: usize) -> usize {
    (r/3*3) + (c/3)
}

fn read_problem(file_name: &str) -> Sudoku {
    let mut file = File::open(file_name).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    Sudoku::from_str(s)
}

fn display_board(board: &Board) {
    for i in board.iter() {
        println!("{:?}", i);
    }
}

fn main() {
    match env::args().nth(1) {
        Some(file) => {
            let mut problem = read_problem(file.as_str());
            println!("Solve {}", file);
            problem.solve();
            display_board(&problem.data);

            // println!("{:?}", problem.test_candidates())
        },
        None => println!("No problem file given.")
    }
}

#[test]
fn test_row_column_to_box() {
    assert_eq!(row_column_to_box(0, 0), 0);
    assert_eq!(row_column_to_box(0, 2), 0);
    assert_eq!(row_column_to_box(0, 3), 1);
    assert_eq!(row_column_to_box(0, 5), 1);
    assert_eq!(row_column_to_box(0, 6), 2);
    assert_eq!(row_column_to_box(0, 8), 2);
    assert_eq!(row_column_to_box(2, 3), 1);
    assert_eq!(row_column_to_box(2, 5), 1);
    assert_eq!(row_column_to_box(2, 6), 2);
    assert_eq!(row_column_to_box(8, 8), 8);
    assert_eq!(row_column_to_box(8, 7), 8);
    assert_eq!(row_column_to_box(7, 7), 8);
    assert_eq!(row_column_to_box(7, 2), 6);
    assert_eq!(row_column_to_box(7, 3), 7);
    assert_eq!(row_column_to_box(6, 1), 6);
    assert_eq!(row_column_to_box(3, 1), 3);
    assert_eq!(row_column_to_box(3, 0), 3);
}

#[test]
fn test_solve_easy() {
    let file_name = "test_data/easy.txt";

    let mut file = File::open(file_name).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let game_strs: Vec<&str> = s.split("\n\n").collect();
    // assert!(false);
    for (c, &game) in game_strs.iter().enumerate() {
        let mut s = Sudoku::from_str(game.to_string().replace("_", "0"));
        println!("=============== {} ===============", c);
        s.solve();
        assert!(s.done());
    }
    // Sudoku::from_str(s)
    // assert!(false);
}
