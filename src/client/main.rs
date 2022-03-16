//todo
//COMPELTE generate more accurate board from actual boggle distribution
//COMPLETE finish chain of user in
//KINDA COMPLETE implement computer "player" (more of a scraper) <-- John's current task
//add timer? need some threading for this so probs too fancy for me for now

mod chars;
mod dice;
mod platform;

use rand::seq::SliceRandom; // 0.7.2

extern crate serde_derive;
use serde_json;

//file imports
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use std::net::TcpStream;

//thread stuff
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time;
extern crate chrono;
extern crate timer;
use std::sync::mpsc::channel;

//use boggle::shared::task::Task;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Previous {
    x: i32,
    y: i32,
}

fn main() {
    //thread::spawn(client);

    // preset stuff
    /*let jsons = load_jsons();

    let board = rand_board();

    print_board(&board);
    //print!("{:?}\n", &board);

    //let scrape: Vec<String> = scraper(&board, &jsons);
    //println!("{:?}", scrape);

    //game loop
    // loop {
    //     let u_in = input(&"Guess a word:\n").to_lowercase();
    //     println!("word: {}", is_word(&u_in, &jsons));
    //     println!("sub: {}", is_sub_word(&u_in, &jsons));
    //     println!("contains: {}", board_contains(&u_in, &board));
    //     print_board(&board);
    //     if check_input(&u_in, &board, &jsons) {
    //         println!("{} is a valid word!", &u_in)
    //     }

    let mut stdin_channel = spawn_stdin_channel("Guess a word:\n");

    let timer = timer::Timer::new();
    let (tx, rx) = channel();


    let _guard = timer.schedule_with_delay(chrono::Duration::seconds(5), move || {
        let _ignored = tx.send(()); // Avoid unwrapping here.
    });

    loop {
        match stdin_channel.try_recv() {
            Ok(key) => got_message(key),
            Err(TryRecvError::Empty) => print!(""),
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {break},
            Err(TryRecvError::Empty) => {}
        }
        sleep(200);
    }*/

    client();
}

fn got_message(inp: String) {
    println!("mesage: {}", inp);
}

fn spawn_stdin_channel(prompt: &'static str) -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        print!("{}", prompt);
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    return rx;
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

fn client() {
    let mut connection = match TcpStream::connect("127.0.0.1:1337") {
        Ok(conn) => conn,
        Err(_) => return,
    };

    println!("{:?}", connection);

    loop {
        let mut buffer = [0; 1000];
        connection
            .read(&mut buffer)
            .expect("Failed to receive message");
        let message = std::str::from_utf8(&buffer).expect("Failed to create string from buffer");
        println!("Received: {}", message);
    }

    /*for x in 0..10 {
        let message = format!("This is message: {}\n", x);
        connection
            .write(message.as_bytes())
            .expect("Failed to send message");


    }*/
}

fn board_contains(word: &String, board: &[[char; 4]; 4]) -> bool {
    for x in 0..4 {
        for y in 0..4 {
            let letter = word[0..1].to_string();
            if board[x][y].to_string().eq(&letter) {
                let prev: Vec<Previous> = Vec::new();
                let res = worm(x, y, letter, &word, &board, &prev);
                if res {
                    return true;
                }
            }
        }
    }
    return false;
}

// fn check_board(word: &String, board: &[[char; 4]; 4]) {
//     let g = 0;
// }

fn worm(
    x: usize,
    y: usize,
    prog: String,
    fin: &String,
    board: &[[char; 4]; 4],
    p: &Vec<Previous>,
) -> bool {
    let mut prev = p.clone();
    prev.push(Previous {
        x: x as i32,
        y: y as i32,
    });
    // println!(
    //     "x: {} y: {} prog: {} fin: {} prev: {:?} ",
    //     &x, &y, &prog, &fin, &prev
    // );
    if &prog == fin {
        return true;
    }
    for relx in 0..3 {
        for rely in 0..3 {
            //println!("relx: {} rely: {}", relx, rely);
            //println!("{}(x) + {}(relx) - 1 = {}", x, relx, x as i32 + relx as i32 - 1);
            let newx: i32 = (x as i32) + (relx as i32) - 1;
            //println!("{}(y) + {}(rely) - 1 = {}", y, rely, y as i32 + rely as i32 - 1);
            let newy: i32 = (y as i32) + (rely as i32) - 1;
            if newx > -1 && newx < 4 && newy > -1 && newy < 4 {
                let newxy = Previous { x: newx, y: newy };
                if !prev.contains(&newxy) {
                    //println!("newx: {} newy: {}", newx, newy);
                    let mut new_prog: String = prog.clone();
                    let throwaway: [char; 4] = board[newx as usize];
                    let throwfarther: char = throwaway[newy as usize];
                    new_prog.push(throwfarther);
                    // println!(
                    //     "prog: {} new_prog: {} fin: {} comp: {}",
                    //     &prog,
                    //     &new_prog,
                    //     &fin,
                    //     substr_compare(&new_prog, &fin)
                    // );
                    if substr_compare(&new_prog, &fin) {
                        //println!("new worm with ({}, {})", newx, newy);
                        let res = worm(newx as usize, newy as usize, new_prog, fin, board, &prev);
                        if res {
                            return true;
                        }
                    }
                }
            }
        }
    }
    return false;
}

fn scraper(board: &[[char; 4]; 4], jsons: &Vec<Vec<String>>) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for x in 0..4 {
        for y in 0..4 {
            let prev: Vec<Previous> = Vec::new();
            let letter = board[x][y].to_string();
            scraper_worm(x, y, letter, &board, &prev, &mut res, &jsons);
        }
    }
    return res;
}

fn scraper_worm(
    x: usize,
    y: usize,
    prog: String,
    board: &[[char; 4]; 4],
    p: &Vec<Previous>,
    res: &mut Vec<String>,
    jsons: &Vec<Vec<String>>,
) {
    let mut prev = p.clone();
    prev.push(Previous {
        x: x as i32,
        y: y as i32,
    });
    //println!("x: {} y: {} prog: {} fin: {} prev: {:?} ", &x, &y, &prog, &fin, &prev);
    for relx in 0..3 {
        for rely in 0..3 {
            //println!("relx: {} rely: {}", relx, rely);
            //println!("{}(x) + {}(relx) - 1 = {}", x, relx, x as i32 + relx as i32 - 1);
            let newx: i32 = (x as i32) + (relx as i32) - 1;
            //println!("{}(y) + {}(rely) - 1 = {}", y, rely, y as i32 + rely as i32 - 1);
            let newy: i32 = (y as i32) + (rely as i32) - 1;
            if newx > -1 && newx < 4 && newy > -1 && newy < 4 {
                let newxy = Previous { x: newx, y: newy };
                if !prev.contains(&newxy) {
                    //println!("newx: {} newy: {}", newx, newy);
                    let mut new_prog: String = prog.clone();
                    let throwaway: [char; 4] = board[newx as usize];
                    let throwfarther: char = throwaway[newy as usize];
                    new_prog.push(throwfarther);
                    //println!("prog: {} new_prog: {} fin: {} comp: {}", &prog, &new_prog, &fin, substr_compare(&new_prog, &fin));
                    if is_sub_word(&new_prog, &jsons) {
                        if !res.contains(&new_prog) {
                            if is_word(&new_prog, &jsons) {
                                println!("Pushing: {}", &new_prog);
                                res.push(new_prog.clone());
                            }
                        }
                        //println!("new worm with ({}, {})", newx, newy);
                        scraper_worm(
                            newx as usize,
                            newy as usize,
                            new_prog,
                            &board,
                            &prev,
                            res,
                            &jsons,
                        );
                    }
                }
            }
        }
    }
}

fn load_jsons() -> Vec<Vec<String>> {
    let mut out_array: Vec<Vec<String>> = Vec::new();
    let mut i: i32 = 0;
    loop {
        let path_str = gen_path(i);
        let path = Path::new(&path_str);
        let dict = json_to_string(path);
        let in_array: Vec<String> = serde_json::from_str(&dict).unwrap();
        out_array.push(in_array);
        i = i + 1;
        if i > 25 {
            break;
        }
    }
    return out_array;
}

//--- FUNCTIONALITY FUNCTIONS ---

//USER INPUT FUNCTIONS
fn input(prompt: &str) -> String {
    print!("{}", prompt);
    let mut user_in = String::new();
    let _cmdbytes = std::io::stdin().read_line(&mut user_in).unwrap();
    return user_in[0..user_in.len() - 2].to_string();
}

fn check_input(inp: &String, board: &[[char; 4]; 4], jsons: &Vec<Vec<String>>) -> bool {
    if input_is_str(inp) {
        if is_word(inp, jsons) {
            if board_contains(inp, board) {
                return true;
            }
        }
    }
    return false;
}

fn input_is_str(inp: &String) -> bool {
    //assumes input is cleaned from \n and \r by input function
    let trimmed = inp.trim().chars();
    for x in trimmed {
        if !(chars::CHARS.contains(&x)) {
            return false;
        }
    }
    return true;
}

fn is_word(word: &String, jsons: &Vec<Vec<String>>) -> bool {
    //assumes input is a string composed of only the 26 lowercase letters
    let res = correct_json(&word, &jsons).iter().position(|x| x.eq(word));
    if res != None {
        return true;
    }
    return false;
}

//JSON FUNCTIONS
fn json_to_string(p: &Path) -> String {
    // Open the file in read-only mode with buffer.
    let mut f = File::open(p).unwrap();
    let mut buffer = String::new();
    let _smth = f.read_to_string(&mut buffer);
    return buffer;
}

fn gen_path(i: i32) -> String {
    let mut path_str: String = String::new();
    path_str.push_str(platform::JSON_PATH);
    path_str.push(chars::CHARS[i as usize]);
    path_str.push_str("subDict.json");
    //println!("{:?}", &path_str);
    return path_str;
}

fn correct_json<'a>(word: &String, json: &'a Vec<Vec<String>>) -> &'a Vec<String> {
    let first: char = word.chars().nth(0).unwrap();
    let ind: usize = chars::CHARS.iter().position(|&x| x == first).unwrap();
    return &json[ind];
}

// fn word(x: i32, y: i32, json: Vec<Vec<String>>) -> String {
//     return String::new()
// }

fn substr_compare(sub: &String, word: &String) -> bool {
    //println!("x = {} || word = {}", &sub.to_string(), &word);
    if sub.len() <= word.len() {
        if word[0..sub.len()].to_string().eq(sub) {
            return true;
        }
    }
    return false;
}

fn is_sub_word(sub: &String, json: &Vec<Vec<String>>) -> bool {
    //println!("{}, {}", &len, &word);
    let res = correct_json(&sub, &json)
        .iter()
        .position(|x| substr_compare(&sub, &x));
    if res != None {
        return true;
    }
    return false;
}

//--- BOARD FUNCTIONS ---
fn rand_board() -> [[char; 4]; 4] {
    let mut b = [['a' as char; 4]; 4];
    let mut i: usize = 0;
    for x in 0..4 {
        for y in 0..4 {
            b[x][y] = *dice::DICE[i].choose(&mut rand::thread_rng()).unwrap();
            i += 1;
        }
    }
    return b;
}

fn _old_rand_board() -> [[char; 4]; 4] {
    let mut b = [['a' as char; 4]; 4];
    for x in 0..4 {
        for y in 0..4 {
            b[x][y] = *chars::CHARS.choose(&mut rand::thread_rng()).unwrap();
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
