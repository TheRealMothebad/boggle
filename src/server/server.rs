use boggle::shared::task::Task;

use std::{
    io::{Read, Write},
    mem::replace,
    net::{TcpListener, TcpStream},
    rc::Rc,
    sync::mpsc,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

use stoppable_thread::{SimpleAtomicBool, StoppableHandle};

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
            Self::listen_loop(stopped, m_receiver);
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

    fn listen_loop(stopped: &SimpleAtomicBool, receiver: Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:1337").expect("Couldn't Bind to Socket");
        listener.set_nonblocking(true).unwrap();

        let mut connections: Vec<Connection> = Vec::new();

        while !stopped.get() {
            match listener.accept() {
                Ok((connection, ..)) => {
                    let (tx, rx) = mpsc::channel();
                    let (has_died_s, has_died_r) = mpsc::channel();
                    connections.push(Connection {
                        handle: stoppable_thread::spawn(move |stopped| {
                            Self::handle_client(stopped, connection, rx, has_died_s)
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

        println!("Breaking out!");

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
        m_receiver: Receiver<String>,
        has_died: Sender<()>,
    ) {
        //handle the client
        while !stopped.get() {
            match m_receiver.try_recv() {
                Ok(message) => {
                    match connection.write(message.as_bytes()) {
                        Ok(_) => {}
                        //kill this thread if we receive an error
                        Err(_) => {
                            has_died.send(()).expect("Failed to send has died");
                            return;
                        }
                    }
                    ();
                }
                Err(_) => {}
            }

            thread::sleep(Duration::from_millis(33));
        }

        println!("Killing connection");
    }
}
