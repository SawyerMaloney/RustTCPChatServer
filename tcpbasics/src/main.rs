use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::io::{stdin,stdout,Write};
use std::sync::{Arc, Mutex};


type Db = Arc<Mutex<Vec<[i32; 1024]>>>;


async fn setup_listener() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("binded to port");
    let mut streams: Vec<TcpStream> = Vec::new(); // tracking our two users

    // wait for our two connections
    while streams.len() < 2 {
        let (socket, _) = listener.accept().await.unwrap();
        handle_connection(socket, &mut streams);
    }

    println!("entering stream loop");

    let db: Db = Arc::new(Mutex::new(Vec::new()));

    for stream in &mut streams {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await {
            Ok(size) => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                if message == "quit" {
                    println!("received quit from client.");
                    break
                }
                println!("received: {}", String::from_utf8_lossy(&buffer[..size]));
                buffer.fill(0);
            },
            Err(e) => eprintln!("Failed to read from stream: {}", e),
        }
    }

    Ok(())
}

fn handle_connection(stream: TcpStream, streams: &mut Vec<TcpStream>) {
    streams.push(stream);
    println!("Exiting handle_connection");
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
