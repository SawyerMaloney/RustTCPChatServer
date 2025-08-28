use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::io::{stdin,stdout,Write};
use tokio::sync::mpsc;


#[derive(Debug)]
struct Message {
    buf: [u8; 1024],
    size: usize
}

impl ToString for Message {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.buf[..self.size]).into_owned()
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

    // mpsc channel to communicate messages
    let (tx, rx) = mpsc::channel::<Message>(128);
    let tx2 = tx.clone();

    let man = tokio::spawn(manager(rx));
    let t1 = tokio::spawn(process(stream1, tx));
    let t2 = tokio::spawn(process(stream2, tx2));

    t1.await.unwrap();
    t2.await.unwrap();
    man.await.unwrap();

    Ok(())
}

async fn process(mut stream: TcpStream, tx: mpsc::Sender<Message>) {
    let mut buf: [u8; 1024] = [0; 1024];
    println!("blocking on stream until readable");
    let size = stream.read(&mut buf).await.unwrap();
    println!("received information and unblocking");
    let msg = Message {
        buf: buf,
        size: size
    };
    tx.send(msg).await.unwrap();
}

async fn manager(mut rx: mpsc::Receiver<Message>) {
    let res = rx.recv().await.unwrap();
    println!("First message, GOT = {:?}", res.to_string());
    let res = rx.recv().await.unwrap();
    println!("Second message, GOT = {:?}", res.to_string());
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
