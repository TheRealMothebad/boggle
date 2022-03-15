mod server;

use std::io::{Write, stdin, stdout};

fn main() -> std::io::Result<()> {
    let server = server::Server::new();

    //listen for input and kill the server if we receive quit
    loop {
        let mut s = String::new();
        print!("Type quit to quit: ");
        stdout().flush()?;
        stdin().read_line(&mut s)?;
        if s.trim_end().eq("quit") {
            server.kill();
            break;
        }
    }

    Ok(())
}
