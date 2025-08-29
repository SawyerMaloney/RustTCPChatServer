use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::io::{stdin,stdout,Write};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use std::mem::size_of;


#[derive(Debug)]
#[derive(Clone)]
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

    fn to_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        v.extend(&self.buf);
        v.extend(self.size.to_be_bytes());
        v.extend(self.sender.to_be_bytes());
        v
    }

    fn from_bytes(v: Vec<u8>) -> Self {
        const USIZE_SIZE: usize = size_of::<usize>();
        const I32_SIZE: usize = size_of::<i32>();

        let mut buf: [u8; 1024] = [0; 1024];
        let mut size: [u8; USIZE_SIZE] = [0; USIZE_SIZE];
        let mut sender: [u8; I32_SIZE] = [0; I32_SIZE];

        for i in 0..buf.len() {
            buf[i] = v[i];
        }

        size.copy_from_slice(&v[buf.len()..buf.len() + USIZE_SIZE]);
        sender.copy_from_slice(&v[buf.len() + USIZE_SIZE..]);

        let size: usize = usize::from_be_bytes(size);
        let sender: i32 = i32::from_be_bytes(sender);

        Self {
            buf,
            size,
            sender
        }
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
    let tx_man = tx1.clone();

    let rx2 = tx1.subscribe();
    let rx_man = tx1.subscribe();

    let man = tokio::spawn(manager(tx_man, rx_man));
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
                // if message is received, parse and send
                let msg = Message {
                    buf: buf,
                    size: size,
                    sender: sender
                };
                let msg_string = msg.to_string() == "quit";
                tx.send(msg).unwrap();
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
                    stream.write(&msg.buf).await.unwrap();
                }
            },
            Err(_) => (),
        };
    }
}

async fn manager(tx: broadcast::Sender<Message>, mut rx: broadcast::Receiver<Message>) {
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
