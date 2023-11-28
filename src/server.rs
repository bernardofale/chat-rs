// Purpose: Main file for the project

use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::io::{Result, Read, Write};
use std::thread::{spawn};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use std::fmt;

enum Message {
    Quit(Arc<TcpStream>),
    NewMessage(Box<Vec<u8>>, SocketAddr),
    Connect(Arc<TcpStream>)
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Quit(_stream) => write!(f, "Client"),
            Message::NewMessage(msg, addr) => write!(f, "Client {addr} sent {:?}", String::from_utf8_lossy(msg.as_ref())),
            Message::Connect(stream) => write!(f, "Client {} connected", stream.as_ref().peer_addr().unwrap()),
        }
    }
}

fn server(rx : Receiver<Message>) -> Result<()> {
    let mut clients = Vec::new();

    loop {
        match rx.recv().unwrap(){
            Message::Connect(stream) => {
                clients.push(stream.clone());
                println!("Client {} connected", stream.as_ref().peer_addr().unwrap());
            },
            Message::NewMessage(msg, addr) => {
                clients.clone().into_iter().filter(|x| x.as_ref().peer_addr().unwrap() != addr).for_each(|x| {
                    x.as_ref().write(msg.as_ref()).unwrap();
                });
                println!("Client {addr} sent {:?}", String::from_utf8_lossy(msg.as_ref()));
            },
            Message::Quit(stream) => {
                    clients.retain(|x| x.as_ref().peer_addr().unwrap() != stream.as_ref().peer_addr().unwrap());
                    println!("Client {} quit from chat", stream.as_ref().peer_addr().unwrap());
                    stream.as_ref().shutdown(Shutdown::Both).unwrap_or_else(|err| {
                        println!("COULD NOT SHUTDOWN CONNECTION : {err}");
                    })
                }
            }
        }
    }

fn handle_conn(stream : Arc<TcpStream>, tx : Sender<Message>) -> Result<()> {
    tx.send(Message::Connect(stream.clone())).unwrap_or_else(|err| {
        println!("COULD NOT SEND MESSAGE TO SERVER : {err}");
    });
    let mut buffer = [0; 128];

    loop{
        let n = stream.as_ref().read(&mut buffer)?;
        let c = Box::new(buffer[0..n].to_vec());
        let q = String::from_utf8(buffer[0..n].to_vec()).unwrap();

        if  q == "quit\r\n".to_string(){
            tx.send(Message::Quit(stream.clone())).unwrap_or_else(|err| println!("COULD NOT QUIT CONNECTION : {err}"));
            return Ok(());
        }else{
            tx.send(Message::NewMessage(c, stream.as_ref().peer_addr().unwrap())).unwrap_or_else(|err| {
                println!("COULD NOT SEND NEW MESSAGE : {err}");
            });
        }
    }
}

fn main() -> Result<()> {

    let addr = "127.0.0.1:420";

    let listener = TcpListener::bind(addr)?;
    let (tx, rx) = channel::<Message>();

    spawn(move || server(rx));

    loop{
        let tx = tx.clone();
        
        match listener.accept() {
            Ok((_socket, _addr)) => {
                spawn(move || {
                    let stream = Arc::new(_socket);
                    handle_conn(stream, tx).unwrap_or_else(|err| {
                        println!("COULD NOT CREATE CLIENT THREAD : {err}");
                    });
                });
            },
            Err(e) => println!("couldn't get client: {e:?}"),
        }
    }
}
