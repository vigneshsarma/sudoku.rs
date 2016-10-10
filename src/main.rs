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
        }//  else {
        //     println!("{:?} {}", self, val)
        // }
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

    fn new_with(&self, r: usize, c: usize, val: u8) -> Sudoku {
        let mut new = Sudoku::new();
        new.set(r, c, val);
        for i in 0..9 {
            for j in 0..9 {
                if self.data[i][j] == 0 {
                    continue
                }
                new.set(i, j, self.data[i][j]);
            }
        }
        new.update_grid();
        new
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
        if self.data[r][c] != 0 {
            println!("{:?} -> {} by {}", (r, c), self.data[r][c], val);
            assert!(false);
        }
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

    fn check_row_possible<F>(&mut self, debug: &str, location_tr: F)
                             -> Result<usize, String>
        where F: Fn((usize, usize)) -> (usize, usize) {
        let mut updates = 0;
        for i in 0..9 { // iter over rows
            let mut recorder:[Vec<usize>; 9] =
                [vec![], vec![], vec![],
                 vec![], vec![], vec![],
                 vec![], vec![], vec![]];

            for j in 0..9 { // iter over columns
                let (r, c) = location_tr((i, j));
                for k in 0..9 { // iter over possible values
                    if self.grid[r][c][k] {
                        let ref mut locations = recorder[k];
                        locations.push(j);
                    }
                }
            }
            for k in 0..9 {
                let ref locations = recorder[k];
                if locations.len() == 1 {
                    let (r, c) = location_tr((i, locations[0]));
                    if self.data[r][c] != 0 {
                        return Err(format!(
                            "Got to bad state with {} {:?} -> {} already has {}",
                            debug, (r, c), k+1, self.data[r][c]))
                    }
                    println!("{} {:?} -> {}", debug, (r, c), k+1);
                    self.set(r, c, (k+1) as u8);
                    updates += 1;
                }
            }
        }
        Ok(updates)
    }

    fn find_loc_with_minimal_possiblities(&self) -> (usize, usize, Vec<u8>) {
        let mut r = 0;
        let mut c = 0;
        let mut min = vec!();
        for i in 0..9 {
            for j in 0..9 {
                if self.data[i][j] != 0 {
                    continue;
                }
                let presense = self.grid[i][j];
                let possible = presense_array_to_vec(&presense);
                if possible.is_empty() {
                    panic!(format!("No possible solution for {:?}", (i, j)));
                } else if possible.len() == 1 {
                    panic!(format!("you have more options before calling me."))
                } else {
                    if possible.len() < 3 {
                        return (i, j, possible)
                    } else {
                        if possible.len() > 0 && (
                            min.len() == 0 || min.len() > possible.len()){
                            r = i;
                            c = j;
                            min = possible;
                        }
                    }
                }
            }
        }
        (r, c, min)
    }

    fn loop_over(&mut self, debug: bool) -> Result<usize, String> {
        let mut updates = 0;
        for i in 0..9 {
            for j in 0..9 {
                if self.data[i][j] != 0 {
                    continue;
                }
                let presense = self.grid[i][j];
                let possible = presense_array_to_vec(&presense);
                if possible.is_empty() {
                    return Err(format!("No possible solution for {:?}", (i, j)));
                } else if  possible.len() == 1 {
                    println!("{:?} -> {:?}", (i, j), possible);
                    self.set(i, j, possible[0]);
                    updates += 1;
                } else if debug {
                    println!("{:?} -> {:?}", (i, j), possible);
                }
            }
        }
        if updates == 0 {
            match self.check_row_possible("Row", |(r, c)| (r, c)) {
                Ok(updates_) => updates = updates_,
                Err(err) => return Err(err)
            }
        }
        if updates == 0 {
            match self.check_row_possible("Column", |(r, c)| (c, r)){
                Ok(updates_) => updates = updates_,
                Err(err) => return Err(err)
            }
        }
        if updates == 0 {
            match self.check_row_possible("Box", |(r, c)| {
                let r_ = ((r/3)*3)+c/3;
                let c_ = ((r%3)*3)+c%3;
                (r_, c_)}) {
                Ok(updates_) => updates = updates_,
                Err(err) => return Err(err)
            }
        }

        Ok(updates)
    }

    fn solve(&mut self) -> Result<Option<Sudoku>, String> {
        while !self.done() {
             match self.loop_over(false) {
                 Ok(updates) => {
                     if updates == 0 {
                         println!("---------- Failed: Cant find any more ---------. {}", self.len);
                         // self.update_grid();
                         // self.loop_over(true).ok();
                         display_board(&self.data);
                         println!("---------- Failed: Cant find any more ---------. {}", self.len);
                         break;
                     } else {
                         self.update_grid();
                         println!("--------------------");
                     }
                 },
                 Err(reason) => return Err(reason)
             }
        }
        if self.done() {
            println!("------- Solved -------------");
            display_board(&self.data);
            println!("------- Solved -------------");
            Ok(None)
        } else {
            let (r, c, options) = self.find_loc_with_minimal_possiblities();
            println!("Now guessing with {:?} -> {:?}", (r, c), options);
            for opt in options.iter() {
                let mut n = self.new_with(r, c, *opt);
                match n.solve() {
                    Err(err) => {
                        println!("Backtracking due to {} for {}", err, opt);
                        continue
                    },
                    Ok(Some(u)) => return Ok(Some(u)),
                    Ok(None) => return Ok(Some(n))
                }
            }
            Err(format!("Cant solve {:?} -> {:?}", (r, c), options))
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
            problem.solve().unwrap();
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
fn test_solve_sudokus() {
    let file_names = ["test_data/easy.txt",
                      "test_data/medium.txt",
                      "test_data/hard.txt"];
    for names in file_names.iter() {
        let mut file = File::open(names).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        let game_strs: Vec<&str> = s.split("\n\n").collect();
        // assert!(false);
        for (c, &game) in game_strs.iter().enumerate() {
            let mut s = Sudoku::from_str(game.to_string().replace("_", "0"));
            println!("=============== {} ===============", c);
            match s.solve() {
                Err(reason) => {
                    println!("{}", reason);
                    assert!(false);
                },
                Ok(Some(n)) => assert!(n.done()),
                Ok(None) => assert!(s.done())
            };
        }
    }
    // Sudoku::from_str(s)
    // assert!(false);
}
