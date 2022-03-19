//todo
//COMPELTE generate more accurate board from actual boggle distribution
//COMPLETE finish chain of user in
//KINDA COMPLETE implement computer "player" (more of a scraper) <-- John's current task
//add timer? need some threading for this so probs too fancy for me for now
//embed in umbriac.com
// https://blog.logrocket.com/rust-webassembly-frontend-web-app-yew/

mod chars;
mod dice;
mod platform;

extern crate serde_derive;
use serde_json;

//async
extern crate async_recursion;
extern crate async_stream;
extern crate tokio;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::timeout;

use std::future::Future;

use std::sync::{Arc, Mutex};

use rand::seq::SliceRandom; // 0.7.2

//file imports
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

//thread stuff
extern crate chrono;
extern crate timer;

//use std::io::stdout;
use std::time::Duration;

use boggle::shared::utils;

#[derive(Debug)]
struct PlayerResult {
    name: String,
    words: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Previous {
    x: i32,
    y: i32,
}

fn main() -> std::io::Result<()> {
    let online = get_online();
    let mut connection: Option<TcpStream> = None;
    let board = if online {
        async_block(async {
            let conn_ref = &mut connection;
            *conn_ref = Some(create_connection().await);
            server_browser(&mut conn_ref.as_mut().unwrap()).await
        })
    } else {
        rand_board()
    };

    print_board(&board);

    let player_result = PlayerResult {
        name: input("What name would you like to use for you results?\n"),
        words: run(&board)
    };

    println!("{:?}", player_result);

    let mut player_results: Vec<PlayerResult> = Vec::new();
    if online {
        async_block(async {
            let conn_ref = &mut connection;
            let player_ref = &mut player_results;
            *player_ref = get_final_state(&mut conn_ref.as_mut().unwrap(), &player_result);
        });
    }
    player_results.push(player_result);

    results(player_results);

    Ok(())
}

fn async_block<F: Future>(block: F) -> F::Output {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(block)
}

fn get_online() -> bool {
    loop {
        let choice = input("offline or online: ");
        if choice.eq("online") {
            return true;
        } else if choice.eq("offline") {
            return false;
        }
    }
}

async fn create_connection() -> TcpStream {
    loop {
        let ip = input("What ip would you like to connect to: ");
        match TcpStream::connect(ip).await {
            Ok(connection) => return connection,
            Err(_) => println!("Could not connect"),
        }
    }
}

async fn server_browser(connection: &mut TcpStream) -> [[char; 4]; 4] {
    loop {
        let command = input("host, join, or list: ");
        let command_args: Vec<&str> = command.split(' ').collect::<Vec<&str>>();

        if command_args[0].eq("host") || command_args[0].eq("join") {
            connection.write(command.as_bytes()).await.unwrap();
            break;
        } else if command_args[0].eq("list") {
            connection.write(command.as_bytes()).await.unwrap();
            let hosts = utils::read_from_connection(connection).await.unwrap();
            println!("hosts: {}", hosts);
        }
    }

    rand_board()
}



/*fn wait_for_response(connection: &TcpStream) -> Option<String> {
    let waiting = stoppable_thread::spawn(|stopped| {
        let conn = connection.clone();
        conn.set_nonblocking(true).unwrap();

        while !stopped.get() {
            match read_string(&conn) {
                Ok(response) => return Some(response),
                Err(_) => {},
            }
            thread::sleep(Duration::from_millis(100));
        }

        conn.set_nonblocking(false).unwrap();

        None
    });

    //while we wait give the user the option to quit
    loop {
        let q = input("type 'quit' or to stop waiting\n");
        if q.eq("quit") {
            waiting.stop.join();
            return None
        }
    }
}*/

fn get_final_state(connection: &mut TcpStream, player_result: &PlayerResult) -> Vec<PlayerResult> {
    Vec::new() 
}

fn run(board: &[[char; 4]; 4]) -> Vec<String> {
    let mut out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let jsons = load_jsons();

    //wrap our asynchronous main loop in a tokio runtime
    async_block(async {
        let clone = Arc::clone(&mut out);
        match timeout(Duration::from_secs(120), main_loop(clone, &board, &jsons)).await {
            Ok(_) => {}  /* should never get here */
            Err(_) => {} /* TODO check if error is not elapsed */
        }
    });

    Arc::try_unwrap(out)
        .expect("Failed to unwrap words")
        .into_inner()
        .expect("Failed to move out of mutex")
}

async fn main_loop(
    words: Arc<Mutex<Vec<String>>>,
    board: &[[char; 4]; 4],
    jsons: &Vec<Vec<String>>,
) {
    loop {
        let input = utils::read_from_stdin("Guess a word:\n")
            .await
            .expect("Failed to read from stdin");
        if check_input(&input, &board, &jsons) {
            words.lock().expect("Failed to get words").push(input);
        }
    }
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

struct GameResult {
    players: Vec<PlayerResult>,
    shared: Vec<String>,
    winner: PlayerResult,
}

fn results(p_res: Vec<PlayerResult>) {
    for word in &p_res[0].words {
        for player in &p_res {
            //if player.words.contains(word)
            let _x = 0;
        }
    }
    //pretty_print(p_res)
}

fn pretty_print(game_result: GameResult) {
    let _x = 0;
}

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
            let newx: i32 = (x as i32) + (relx as i32) - 1;
            let newy: i32 = (y as i32) + (rely as i32) - 1;
            if newx > -1 && newx < 4 && newy > -1 && newy < 4 {
                let newxy = Previous { x: newx, y: newy };
                if !prev.contains(&newxy) {
                    let mut new_prog: String = prog.clone();
                    let throwaway: [char; 4] = board[newx as usize];
                    let throwfarther: char = throwaway[newy as usize];
                    new_prog.push(throwfarther);
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
    for relx in 0..3 {
        for rely in 0..3 {
            let newx: i32 = (x as i32) + (relx as i32) - 1;
            let newy: i32 = (y as i32) + (rely as i32) - 1;
            if newx > -1 && newx < 4 && newy > -1 && newy < 4 {
                let newxy = Previous { x: newx, y: newy };
                if !prev.contains(&newxy) {
                    let mut new_prog: String = prog.clone();
                    let throwaway: [char; 4] = board[newx as usize];
                    let throwfarther: char = throwaway[newy as usize];
                    new_prog.push(throwfarther);
                    if is_sub_word(&new_prog, &jsons) {
                        if !res.contains(&new_prog) {
                            if is_word(&new_prog, &jsons) {
                                println!("Pushing: {}", &new_prog);
                                res.push(new_prog.clone());
                            }
                        }
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
    std::io::stdout().flush().expect("Couldn't flush");
    let mut user_in = String::new();
    let _cmdbytes = std::io::stdin().read_line(&mut user_in).unwrap();
    return user_in.trim().to_string();
}

fn check_input(inp: &String, board: &[[char; 4]; 4], jsons: &Vec<Vec<String>>) -> bool {
    if input_is_str(inp) {
        if is_word(inp, jsons) {
            if board_contains(inp, board) {
                println!("Found word");
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
