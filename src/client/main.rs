//todo
//generate more accurate board from boggle distribution
//finish chain of user in
//implement computer player

use rand::seq::SliceRandom; // 0.7.2

extern crate serde_derive;
use serde_json;

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Previous {
    x: i32,
    y: i32,
}

fn main() {
    println!("Hello, world!");
    // preset stuff
    let chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

    let jsons = load_jsons(&chars);

    let board = rand_board(&chars);

    print_board(&board);
    print!("{:?}\n", &board);

    //game loop
    loop {
        let u_in = input(&"Guess a word:\n");
        println!("word: {}", check_word(&u_in, &jsons));
        println!("sub: {}", check_sub(&u_in, &jsons));
        println!("contains: {}", contains_word(&u_in, &board))

    }
}

fn contains_word(word: &String, board: &[[char; 4]; 4]) -> bool {
    for x in 0..4 {
        for y in 0..4 {
            let letter = word[0 .. 1].to_string();
            if board[x][y].to_string().eq(&letter) {
                let prev: Vec<Previous> = Vec::new();
                let res = worm(x, y, letter, &word, &board, &prev);
                if res {
                    return true
                }
            }
        }
    }
    return false
}

// fn check_board(word: &String, board: &[[char; 4]; 4]) {
//     let g = 0;
// }

fn worm(x: usize, y: usize, prog: String, fin: &String, board: &[[char; 4]; 4], p: &Vec<Previous>) -> bool {
    let mut prev = p.clone();
    prev.push(Previous {x: x as i32, y: y as i32});
    println!("x: {} y: {} prog: {} fin: {} prev: {:?} ", &x, &y, &prog, &fin, &prev);
    if &prog == fin {
        return true;
    }
    for relx in 0 .. 3 {
        for rely in 0 .. 3 {
            //println!("relx: {} rely: {}", relx, rely);
            //println!("{}(x) + {}(relx) - 1 = {}", x, relx, x as i32 + relx as i32 - 1);
            let newx: i32 = (x as i32) + (relx as i32) - 1;
            //println!("{}(y) + {}(rely) - 1 = {}", y, rely, y as i32 + rely as i32 - 1);
            let newy: i32 = (y as i32) + (rely as i32) - 1;
            if newx > -1 && newx < 4 && newy > -1 && newy < 4 {
                let newxy = Previous {x : newx, y : newy};
                if !prev.contains(&newxy) {
                    println!("newx: {} newy: {}", newx, newy);
                    let mut new_prog: String = prog.clone();
                    let throwaway: [char; 4] = board[newx as usize];
                    let throwfarther: char = throwaway[newy as usize];
                    new_prog.push(throwfarther);
                    println!("prog: {} new_prog: {} fin: {} comp: {}", &prog, &new_prog, &fin, substr_compare(&new_prog, &fin));
                    if substr_compare(&new_prog, &fin) {
                        println!("new worm with ({}, {})", newx, newy);
                        let res = worm(newx as usize, newy as usize, new_prog, fin, board, &prev);
                        if res {
                            return true;
                        }
                    }
                }
            }
        }
    }
    return false
}

fn load_jsons(chars: &[char; 26]) -> Vec<Vec<String>> {
    let mut out_array: Vec<Vec<String>>= Vec::new();
    let mut i: i32 = 0;
    loop {
        let path_str = gen_path(i, chars);
        let path = Path::new(&path_str);
        let dict = json_to_string(path);
        let in_array: Vec<String> = serde_json::from_str(&dict).unwrap();
        out_array.push(in_array);
        i = i + 1;
        if i > 25 {
            break;
        }
    }
    return out_array
}

//--- FUNCTIONALITY FUNCTIONS ---
fn input(prompt: &str) -> String {
    print!("{}", prompt);
    let mut user_in = String::new();
    let _cmdbytes = std::io::stdin().read_line(&mut user_in).unwrap();
    return user_in[0 .. user_in.len() - 2].to_string();
}

fn json_to_string(p: &Path) -> String {
    // Open the file in read-only mode with buffer.
    let mut f = File::open(p).unwrap();
    let mut buffer = String::new();
    let _smth = f.read_to_string(&mut buffer);
    return buffer;
}

fn gen_path(i: i32, chars: &[char; 26]) -> String {
    let mut path_str: String = String::new();
    path_str.push_str("/home/swyngaard/Documents/projects/boggle/src/client/sub_dicts/");
    path_str.push(chars[i as usize]);
    path_str.push_str("subDict.json");
    println!("{:?}", &path_str);
    return path_str;
}

fn correct_json<'a>(word: &String, json: &'a Vec<Vec<String>>) -> &'a Vec<String> {
    let chars = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];
    let first: char = word.chars().nth(0).unwrap();
    let ind: usize = chars.iter().position(|&x| x == first).unwrap();
    return &json[ind];
}

// fn word(x: i32, y: i32, json: Vec<Vec<String>>) -> String {
//     return String::new()
// }

fn check_word (word: &String, json: &Vec<Vec<String>>) -> bool {
    let res = correct_json(&word, &json).iter().position(|x| x.eq(word));
    if res != None {
        return true
    }
    return false;
}

fn substr_compare(sub: &String, word: &String) -> bool {
    //println!("x = {} || word = {}", &sub.to_string(), &word);
    if sub.len() <= word.len() {
        if word[0 .. sub.len()].to_string().eq(sub) {
            return true
        }
    }
    return false
}

fn check_sub (sub: &String,  json: &Vec<Vec<String>>) -> bool {
    //println!("{}, {}", &len, &word);
    let res = correct_json(&sub, &json).iter().position(|x| substr_compare(&sub, &x));
    if res != None {
        return true
    }
    return false;
}

//--- BOARD FUNCTIONS ---
fn rand_board(chars: &[char; 26]) -> [[char; 4]; 4] {
    let mut b = [['a' as char; 4]; 4];

    for x in 0..4 {
        for y in 0..4 {
            b[x][y] = *chars.choose(&mut rand::thread_rng()).unwrap();
        }
    }
    return b;
}

fn print_board(b: &[[char; 4]; 4]) {
    //debug
    //print!("{:?}", b);
    for y in 0..4 {
        for x in 0..4 {
            print!("{}", b[x][y]);
            if x != 3 {
                print!(" ");
            }
        }
    println!("");
    }
}
