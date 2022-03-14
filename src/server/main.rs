use std::{
    io::{stdout, Read, Write, stdin},
    net::{TcpListener, TcpStream},
    sync::mpsc::{TryRecvError, Receiver},
    thread,
    time::Duration,
};

use boggle::shared::task::Task;

fn handle_client(mut connection: TcpStream, rx: Receiver<bool>) {
    //handle the client
    loop {
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        let mut message: String = String::from("");
        connection
            .read_to_string(&mut message)
            .expect("Error receiving");
        print!("{}", message);

        thread::sleep(Duration::from_millis(33));
    }

    println!("Killing connection");
}

fn server(rx: Receiver<bool>) {
    let listener = TcpListener::bind("127.0.0.1:1337").expect("Couldn't Bind to Socket");
    listener.set_nonblocking(true).unwrap();

    let mut connections: Vec<Task> = Vec::new();

    loop {
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                println!("Killing server");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        match listener.accept() {
            Ok((connection, ..)) => {
                connections.push(Task::new(move |receiver| {
                    handle_client(connection, receiver);
                }));
            }
            Err(..) => continue,
        };

        thread::sleep(Duration::new(1, 0));
    }

    //consume connections and kill them
    for conn in connections.into_iter() {
        conn.kill();
    }
}

fn main() -> std::io::Result<()> {
    let serv: Task = Task::new(|receiver| {
        server(receiver);
    });

    println!("Getting here");

    //listen for input and kill the server if we receive quit
    loop {
        let mut s = String::new();
        print!("Type quit to quit: ");
        stdout().flush()?;
        stdin().read_line(&mut s)?;
        if s.trim_end().eq("quit") {
            serv.kill();
            break;
        }
    }

    Ok(())
}
