extern crate tokio;
extern crate async_recursion;

use tokio::select;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::sync::oneshot::channel;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;

use boggle::shared::utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1337").await?;

    let mut connections: Vec<JoinHandle<()>> = Vec::new();
    let mut kill_connections: Vec<Sender<()>> = Vec::new();

    loop {
        select! {
            Ok((connection, _)) =  listener.accept() => {
                let (sender, receiver): (Sender<()>, Receiver<()>) = channel();

                kill_connections.push(sender);
                connections.push(tokio::spawn(async {
                    select! {
                        _ = handle_client(connection) => {
                            println!("Some error in client execution");
                        },
                        _ = receiver => {
                            println!("Aborting the connection");
                        }
                    }
                }));
            }
            _ = utils::read_quit_continue() => {
                break
            }
        }
    }

    for conn in kill_connections {
        conn.send(()).expect("Failed to send on mpsc channel");
    }

    for conn in connections {
        conn.await.expect("Connection returned with an error");
    }

    Ok(())
}

async fn handle_client(mut stream: TcpStream){
    println!("Handling new connection");
    loop {
        let input = match utils::read_from_connection(&mut stream).await { 
            Ok(input) => input,
            Err(_) => {
                println!("Received some error from socket");
                break
            }
        };

        println!("Got message {} from {:?}", input, stream);
    }
}
