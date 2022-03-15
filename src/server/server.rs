use boggle::shared::task::Task;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

use stoppable_thread::{SimpleAtomicBool, StoppableHandle};

struct Connection {
    handle: StoppableHandle<()>,
    sender: Sender<String>,
}

pub struct Server {
    m_sender: Sender<String>,
    handle: Option<StoppableHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        let (m_sender, m_receiver) = mpsc::channel();

        let mut instance = Self {
            m_sender,
            handle: None,
        };

        instance.handle = Some(stoppable_thread::spawn(|stopped| {
            Self::listen_loop(stopped, m_receiver);
        }));

        instance
    }

    pub fn send(&self, message: &str) {
        self.m_sender.send(message.to_string());
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
                    connections.push(Connection {
                        handle: stoppable_thread::spawn(move |stopped| {
                            Self::handle_client(stopped, connection, rx)
                        }),
                        sender: tx,
                    });
                }
                Err(..) => continue,
            };

            //pass all the messages we receive to the connections
            match receiver.try_recv() {
                Ok(message) => {
                    for conn in &connections {
                        conn.sender
                            .send(message.clone())
                            .expect("Failed to send message to connection");
                    }
                    ()
                }
                Err(TryRecvError::Disconnected) => break,
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
        m_receiver: Receiver<String>,
    ) {
        //handle the client
        while !stopped.get() {
            let mut message: String = String::from("");
            connection
                .read_to_string(&mut message)
                .expect("Error receiving");
            print!("{}", message);

            match m_receiver.try_recv() {
                Ok(message) => {
                    connection
                        .write(message.as_bytes())
                        .expect("Failed to send message!");
                    return ();
                }
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }

            thread::sleep(Duration::from_millis(33));
        }

        println!("Killing connection");
    }
}
