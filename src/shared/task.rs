use std::{
    sync::mpsc::{Sender, Receiver},
    sync::mpsc,
    thread::JoinHandle,
    thread,
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
