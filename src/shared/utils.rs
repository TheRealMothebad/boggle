extern crate tokio;
extern crate async_recursion;

use async_recursion::async_recursion;

use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::stdin;
use tokio::io::stdout;

#[async_recursion]
pub async fn read_quit_continue() {
    let input = read_from_stdin("Type 'quit' to quit!\n").await.unwrap();
    if input.eq("quit") {
        return;
    }
    read_quit_continue().await
}

pub async fn read_from_stdin(prompt: &str) -> std::io::Result<String> {
    stdout().write(prompt.as_bytes()).await?;
    stdout().flush().await?;
    let mut buffer = [0 as u8; 255];
    let length = stdin().read(&mut buffer).await?;
    let message = std::str::from_utf8(&buffer[..length])
        .expect("Couldn't convert message to buffer!")
        .trim()
        .to_string();
    Ok(message)
}

pub async fn read_from_connection(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buffer = [0 as u8; 10000];
    let length = stream.read(&mut buffer).await?;
    let message = std::str::from_utf8(&buffer[..length])
        .expect("Couldn't convert message to buffer!")
        .to_string();
    //println!("Message has length: {}", message.len());
    Ok(message)
}
