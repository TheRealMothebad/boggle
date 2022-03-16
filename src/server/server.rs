use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

use stoppable_thread::{SimpleAtomicBool, StoppableHandle};

//listeners view of a connection
struct Connection {
    handle: StoppableHandle<()>,
    sender: Sender<String>,
    has_died: Receiver<()>,
}

pub struct Server {
    m_sender: Sender<String>,
    handle: Option<StoppableHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        let (m_sender, m_receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

        let mut instance = Self {
            m_sender,
            handle: None,
        };

        instance.handle = Some(stoppable_thread::spawn(move |stopped| {
            Self::handler(stopped, m_receiver);
        }));

        instance
    }

    pub fn send(&self, message: &str) {
        self.m_sender
            .send(message.to_string())
            .expect("Failed to send message to listener");
    }

    pub fn kill(self) {
        self.handle.unwrap().stop().join().expect("Failed to join");
    }

    fn handler(stopped: &SimpleAtomicBool, receiver: Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:1337").expect("Couldn't Bind to Socket");
        listener.set_nonblocking(true).unwrap();

        let mut connections: Vec<Connection> = Vec::new();
        let hosts: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        while !stopped.get() {
            match listener.accept() {
                Ok((connection, ..)) => {
                    let (tx, rx) = mpsc::channel();
                    let (has_died_s, has_died_r) = mpsc::channel();
                    let h = Arc::clone(&hosts);
                    connections.push(Connection {
                        handle: stoppable_thread::spawn(move |stopped| {
                            Self::handle_client(stopped, connection, h, has_died_s)
                        }),
                        sender: tx,
                        has_died: has_died_r,
                    });
                }
                Err(..) => {}
            };

            //check if any of the connections have died and join them
            let mut to_remove = Vec::new();
            for (i, conn) in connections.iter().enumerate() {
                match conn.has_died.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        to_remove.push(i);
                    }
                    Err(_) => {}
                }
            }

            for i in to_remove {
                println!("Removing connection!");
                connections.remove(i).handle.join().expect("Failed to join");
            }

            match receiver.try_recv() {
                Ok(message) => {
                    for conn in &connections {
                        conn.sender
                            .send(message.clone())
                            .expect("Failed to send message to connection");
                    }
                    ()
                }
                Err(TryRecvError::Disconnected) => println!("disconnected"),
                Err(TryRecvError::Empty) => {}
            }

            thread::sleep(Duration::new(1, 0));
        }

        //consume connections and kill them
        for conn in connections.into_iter() {
            conn.handle
                .stop()
                .join()
                .expect("Failed to join connection handler");
        }
    }

    fn handle_client(
        stopped: &SimpleAtomicBool,
        mut connection: TcpStream,
        hosts_lock: Arc<Mutex<Vec<String>>>,
        has_died: Sender<()>,
    ) {
        //handle the client
        loop {
            let command = read_string(&mut connection).unwrap();
            println!("command: {}", command);
            let command_args: Vec<&str> = command.split(' ').collect::<Vec<&str>>();
            if command_args[0].eq("host") {
                let mut hosts = hosts_lock.lock().unwrap();
                if command_args.len() > 1 {
                    hosts.push(command_args[1].to_string());
                    println!("hosting {}", command_args[1]);
                    //break
                }
            } else if command_args[0].eq("join") {
                println!("The client would like to join!")
            } else if command.eq("list") {
                let hosts = hosts_lock.lock().unwrap();
                println!("listing: {}", hosts.join(" "));
                if hosts.len() > 0 {
                    connection.write(hosts.join(" ").as_bytes()).unwrap();
                } else {
                    connection.write(b"[NONE]").unwrap();
                }
            }
        }

        println!("Killing connection");
    }
}

fn read_string(mut connection: &TcpStream) -> std::io::Result<String> {
    let mut buffer = [0 as u8; 10000];
    let length = connection.read(&mut buffer)?;
    Ok(std::str::from_utf8(&buffer[..length]).unwrap().to_string())
}
