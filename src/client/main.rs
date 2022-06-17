//todo
//COMPELTE generate more accurate board from actual boggle distribution
//COMPLETE finish chain of user in
//KINDA COMPLETE implement computer "player" (more of a scraper) <-- John's current task
//add timer? need some threading for this so probs too fancy for me for now
//embed in umbriac.com
// https://blog.logrocket.com/rust-webassembly-frontend-web-app-yew/

mod chars;
mod dice;
mod dict;

use std::io::Write;

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

    //unit_tests();

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

    let player_result = PlayerResult {
        name: input("What name would you like to use for your results?\n"),
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

fn get_final_state(connection: &mut TcpStream, player_result: &PlayerResult) -> Vec<PlayerResult> {
    Vec::new()
}

fn run(board: &[[char; 4]; 4]) -> Vec<String> {
    let mut out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // putting this here so it appears after the name prompt to avoid confusion
    print_board(&board);

    //wrap our asynchronous main loop in a tokio runtime
    async_block(async {
        let clone = Arc::clone(&mut out);
        match timeout(Duration::from_secs(120), main_loop(clone, &board)).await {
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
) {
    loop {
        let input = utils::read_from_stdin("Guess a word:\n")
            .await
            .expect("Failed to read from stdin");
        if check_input(&input, &board) {
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

fn unit_tests() {
    print_test();
}

fn print_test() {
    let mut players: Vec<PlayerResult> = Vec::new();
    players.push(PlayerResult { name: "jeff".to_string(), words: vec!["this".to_string(), "is".to_string(), "a".to_string(), "test".to_string()]});
    players.push(PlayerResult { name: "jeff".to_string(), words: vec!["this".to_string(), "is".to_string(), "a".to_string(), "stick".to_string(), "up".to_string()]});
    results(players)
}

#[derive(Debug)]
struct GameResult<'a> {
    players: &'a Vec<PlayerResult>,
    shared: Vec<String>,
    winner: &'a PlayerResult,
}

fn results(p_res: Vec<PlayerResult>) {
    let gr: GameResult = GameResult {
        players: &p_res,
        shared: calc_shared(&p_res),
        winner: calc_winner(&p_res)
    };
    pretty_print(gr);
}

fn calc_shared(p_res: &Vec<PlayerResult>) -> Vec<String> {
    let mut shared: Vec<String> = Vec::new();
    for word in &p_res[0].words {
        let mut cont = true;
        for player in p_res {
            if !player.words.contains(word) {
                cont = false;
            }
        }
        if cont {
            shared.push(word.to_string())
        }
    }
    shared
}

fn calc_winner(players: &Vec<PlayerResult>) -> &PlayerResult {
    let mut winner: &PlayerResult = &players[0];
    for x in players {
        if x.words.len() > winner.words.len() {
            winner = x;
        }
    }
    winner
}

fn greatest_len_word(vect: &Vec<PlayerResult>) -> usize {
    let mut greatest: usize = 0;
    for x in vect {
        for i in &x.words {
            if i.chars().count() > greatest {
                greatest = i.chars().count()
            }
        }
    }
    greatest
}

fn pretty_print(gr: GameResult) {
    println!("Results:\n{} won!\n\nWords Found:\n", &gr.winner.name);
    let word_width: usize = greatest_len_word(&gr.players) + 1;
    print_shared_words(&gr, word_width);
    //println!("ending printing of shared words");
    let mut nexts: Vec<usize> = Vec::new();
    for word_list in gr.players {
        nexts.push(0)
    }
    loop {
        for index in 0 .. gr.players.len() {
            let mut cur_word: &String = &"".to_string();
            let mut will_print = true;

            loop {
                //println!("got here with index: {}", index);
                if nexts[index] >= gr.players[index].words.len() {
                    will_print = false;
                    break;
                }
                if gr.shared.contains(&gr.players[index].words[nexts[index]]) {
                    nexts[index] += 1;
                }
                else {
                    break;
                }
            }
            if will_print {
                cur_word = &gr.players[index].words[nexts[index]]
            }

            print!("{wrd:<wid$}", wrd=cur_word, wid=word_width);

            nexts[index] += 1;
        }
        println!("");
        let mut done = true;
        for i in 0 .. nexts.len() {
            //print!("nexts[{}] is {}", i, nexts[i]);
            if nexts[i] <= gr.players[i].words.len() {
                done = false;
            }
        }
        if done {
            break;
        }

    }
}



fn print_shared_words(game_result: &GameResult, word_width: usize) {
    //println!("{:?}", game_result);
    for word in &game_result.shared {
        for _player in game_result.players {
            print!("{wrd:<wid$}", wrd=word, wid=word_width)
        }
        println!("");
    }
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

fn scraper(board: &[[char; 4]; 4]) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for x in 0..4 {
        for y in 0..4 {
            let prev: Vec<Previous> = Vec::new();
            let letter = board[x][y].to_string();
            scraper_worm(x, y, letter, &board, &prev, &mut res);
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
                    if is_sub_word(&new_prog) {
                        if !res.contains(&new_prog) {
                            if is_word(&new_prog) {
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
                        );
                    }
                }
            }
        }
    }
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

fn check_input(inp: &String, board: &[[char; 4]; 4]) -> bool {
    if input_is_str(inp) {
        if is_word(inp) {
            if board_contains(inp, board) {
                println!("$$ | Word Found! | $$");
                return true;
            }
            else {
                println!("?? | Word not found on board | ??")
            }
        }
        else {
            println!("?? | Word not recognised | ??")
        }
    }
    else {
        println!("XX | Input should only contain letters | XX", );
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

fn is_word(word: &String) -> bool {
    //assumes input is a string composed of only the 26 lowercase letters
    match dict::DICT.get(&word as &str) {
        Some(b) => return *b,
        None => return false
    }
}

//DICT FUNCTIONS
fn substr_compare(sub: &String, word: &String) -> bool {
    //println!("x = {} || word = {}", &sub.to_string(), &word);
    if sub.len() <= word.len() {
        if word[0..sub.len()].to_string().eq(sub) {
            return true;
        }
    }
    return false;
}

fn is_sub_word(sub: &String) -> bool {
    //println!("{}, {}", &len, &word);
    match dict::DICT.get(&sub as &str) {
        Some(b) => return !*b,
        None => return false
    }
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
