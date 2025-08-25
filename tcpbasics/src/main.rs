use std::net::{TcpListener, TcpStream};
// use std::thread;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::io::{stdin,stdout,Write};


fn setup_listener() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    println!("binded to port");

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            },
            Err(_) => (),
        }
    }

    println!("Finished in listener");

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(size) => {
            println!("received: {}", String::from_utf8_lossy(&buffer[..size]));
        },
        Err(e) => eprintln!("Failed to read from stream: {}", e),
    }
}

fn setup_client() -> std::io::Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
    let mut stream = TcpStream::connect(&socket)?;
    println!("connected to stream");

    let mut message = String::new();
    print!("Enter your message: ");
    let _ = stdout().flush();
    stdin().read_line(&mut message).expect("Did not enter valid string");
    let bytes_written = stream.write(message.as_bytes())?;

    println!("Bytes written: {}", bytes_written);
    println!("Finished in client.");
    Ok(())
}

fn main() -> std::io::Result<()> {
    // println!("Spawning listener handle");
    // let listener_handle = thread::spawn(|| -> std::io::Result<()> {
    //     setup_listener()
    // });

    // println!("Spawning client handle");
    // let client_handle = thread::spawn(|| -> std::io::Result<()> {
    //     setup_client()
    // });

    // println!("Joining on listener handle");
    // match listener_handle.join() {
    //     Ok(_) => (),
    //     Err(_) => println!("Problem in listener handle"),
    // };
    // println!("Joining on client handle");
    // match client_handle.join() {
    //     Ok(_) => (),
    //     Err(_) => println!("Problem in client handle"),
    // };

    let mut choice = String::new();
    print!("Would you like to be (c)lient or (s)erver: ");
    let _ = stdout().flush();
    stdin().read_line(&mut choice).expect("Failed to read line.");
    
    if choice.trim() == "c" {
        println!("Starting client.");
        let _ = setup_client();
    }
    else if choice.trim() == "s" {
        println!("Starting server.");
        let _ = setup_listener();
    }
    else {
        println!("Please enter 'c' or 's'.");
    }

    Ok(())
}