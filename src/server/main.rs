use std::{
    io::{stdout, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    sync::mpsc::{Sender, Receiver, TryRecvError},
    thread,
    thread::JoinHandle,
    time::Duration,
};

pub struct Task {
    sender: Sender<bool>,
    handle: JoinHandle<()>,
}

impl Task {
    pub fn new<F>(function: F) -> Self
    where
        F: FnOnce(Receiver<bool>),
        F: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            function(receiver);
        });

        Self { sender, handle }
    }

    pub fn kill(self) {
        self.sender.send(true).unwrap();
        self.handle.join().unwrap();
    }
}

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
    println!("Broke out");

    //consume connections and kill them
    for conn in connections.into_iter() {
        conn.kill();
    }
}

fn main(){
    let serv: Task = Task::new(|receiver| {
        server(receiver);
    });


}
