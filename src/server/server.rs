use boggle::shared::task::Task;

use std::{
    sync::mpsc::{Sender, Receiver, TryRecvError},
    sync::{mpsc, atomic::AtomicBool},
    net::{TcpListener, TcpStream},
    thread,
    thread::JoinHandle,
    io::Read,
};

//use stoppable_thread;

pub struct Server {
    m_sender: Sender<String>,
    handle: Option<JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        let (m_sender, m_receiver) = mpsc::channel();
        
        let instance = Self {m_sender, handle: None};

        instance.handle = stoppable_thread::spawn(|stopped| {
            Self::listen_loop(stopped, m_receiver);
        });

        instance
    }

    pub fn send(&self, message: &str) {
        self.m_sender.send(message.to_string());
    }

    pub fn kill(&self) {
        self.handle.unwrap().stop().join();
    }

    fn listen_loop(stopped: AtomicBool, receiver: Receiver<String>){
        
        let listener = TcpListener::bind("127.0.0.1:1337").expect("Couldn't Bind to Socket");
        listener.set_nonblocking(true).unwrap();

        let mut connections: Vec<Task> = Vec::new();

        while !stopped.get() {
            match listener.accept() {
                Ok((connection, ..)) => {
                    connections.push(Task::new(move |receiver| {
                        Self::handle_client(connection, receiver);
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
}
