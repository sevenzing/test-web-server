use std::{
    net::{TcpListener, TcpStream},
    thread,
};

use crossbeam_channel::{bounded, select, Receiver};
use std::error::Error;
use web_server::{handle_connection, ThreadPool};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn stream_channel(listener: TcpListener) -> Result<Receiver<TcpStream>, Box<dyn Error>> {
    let (sender, receiver) = bounded(100);
    thread::spawn(move || {
        for stream in listener.incoming() {
            sender.send(stream.unwrap()).unwrap();
        }
    });
    Ok(receiver)
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:6655").unwrap();
    let pool = ThreadPool::new(4);

    let ctrl_c_events = ctrl_channel()?;
    let stream_events = stream_channel(listener)?;

    loop {
        select! {
            recv(stream_events) -> stream => {
                pool.execute(move || {
                    handle_connection(stream.unwrap());
                })
            }
            recv(ctrl_c_events) -> _ => {
                break;
            }
        }
    }
    Ok(())
}
