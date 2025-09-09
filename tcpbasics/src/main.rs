use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::io::{stdin,stdout,Write};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use std::mem::size_of;

// axum framework
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router
};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    buf: [u8; 1024],
    size: usize,
    sender: i32
}

impl ToString for Message {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.buf[..self.size]).into_owned()
    }
}

impl Message {
    fn get_sender(&self) -> String {
        self.sender.to_string()
    }
}


async fn setup_listener() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("binded to port");
    let stream1: TcpStream;
    let stream2: TcpStream;

    println!("Waiting for first connection...");
    let (socket, _) = listener.accept().await.unwrap();
    stream1 = socket;

    println!("Received first connection. Waiting for second connection...");

    let (socket, _) = listener.accept().await.unwrap();
    stream2 = socket;


    println!("Both users connected. Going to stream listening loop...");

    // broadcast channel to communicate messages in both directions
    let (tx1, rx1) = broadcast::channel::<Message>(128);
    // set up clones of handles
    let tx2 = tx1.clone();

    let rx2 = tx1.subscribe();
    let rx_man = tx1.subscribe();

    let man = tokio::spawn(manager(rx_man));
    let t1 = tokio::spawn(client_process(stream1, tx1, rx1, 0));
    let t2 = tokio::spawn(client_process(stream2, tx2, rx2, 1));

    t1.await.unwrap();
    t2.await.unwrap();
    man.await.unwrap();

    Ok(())
}

async fn client_process(mut stream: TcpStream, tx: broadcast::Sender<Message>, mut rx: broadcast::Receiver<Message>, sender: i32) {
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        // try to read from TcpStream 
        match stream.try_read(&mut buf) {
            Ok(size) => {
                println!("try_read succeeded on sender {}", sender);
                // if message is received, parse and send
                let msg = Message {
                    buf: buf,
                    size: size,
                    sender: sender
                };
                let msg_string = msg.to_string() == "quit";
                match tx.send(msg) {
                    Ok(size) => println!("Broadcasted bytes: {}", size),
                    Err(_) => {
                        println!("Error while broadcasting in client_process");
                    },
                };
                if msg_string {
                    break;
                }
            },
            Err(_) => {
                // no tcp was found
                ()
            }
        };

        // try to read from broadcast
        match rx.try_recv() {
            Ok(msg) => {
                // message received from broadcast
                // Check if new message, propogate to TcpStream
                if msg.sender != sender {
                    println!("client_process: received message on broadcast. Sending to TcpStream.");
                    stream.write(&bincode::serialize(&msg).unwrap()).await.unwrap();
                }
            },
            Err(_) => (),
        };
    }
}

async fn manager(mut rx: broadcast::Receiver<Message>) {
    println!("Started manager,");
    loop {
        let _ = match rx.recv().await {
            Ok(msg) => {
                println!("received message, {}, from sender {}", msg.to_string(), msg.get_sender());
                msg
            },
            Err(RecvError::Closed) => {
                println!("All sender halves have dropped");
                break
            },
            Err(_) => {
                println!("unimplemented");
                break;
            },
        };
    }
    println!("broken from loop");
}


async fn setup_client() -> std::io::Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
    let mut stream = TcpStream::connect(&socket).await.unwrap();
    println!("connected to stream");

    let mut message = String::new();
    while message != "quit\n" {
        message.clear();
        print!("Enter your message, or 'quit': ");
        match stdout().flush() {
            Ok(()) => (),
            Err(e) => eprintln!("Could not flush stdout. Panic'd with error {e}"),
        };
        stdin().read_line(&mut message).expect("Did not enter valid string");
        let bytes_written = stream.write(message.trim().as_bytes()).await?;

        println!("Bytes written: {}", bytes_written);

        // DEBUG TODO remove, check TcpStream for incoming 
        let mut buf: [u8; 2048] = [0; 2048];
        match stream.try_read(&mut buf) {
            Ok(_) => {
                let mut v = Vec::new();
                v.extend(buf);
                let msg: Message = bincode::deserialize(v);
                println!("Received message on TcpStream, {}, from sender {}", msg.to_string(), msg.sender);
            },
            Err(_) => (),
        };
    }
    println!("Finished in client.");
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let mut choice = String::new();
    print!("Would you like to be (c)lient or (s)erver: ");
    let _ = stdout().flush();
    stdin().read_line(&mut choice).expect("Failed to read line.");
    
    if choice.trim() == "c" {
        println!("Starting client.");
        let _ = setup_client().await;
    }
    else if choice.trim() == "s" {
        println!("Starting server.");
        let _ = setup_listener().await;
    }
    else {
        println!("Please enter 'c' or 's'.");
    }

    Ok(())
}