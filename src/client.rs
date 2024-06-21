
use std::{net::{SocketAddr, ToSocketAddrs}, sync::Arc, time::Duration, error::Error};
use tokio::{sync::{mpsc, oneshot}, time::timeout};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use colored::Colorize;


const MAX_DATAGRAM_SIZE: usize = 65535;

enum Command {
    Receive {
        responder: Responder<String>
    },
    Send {
        msg: String,
        responder: Responder<usize>,
    }
}

type Responder<T> = oneshot::Sender<Result<T, Box<dyn Error + Send + Sync>>>;

pub async fn start(server_addr: String, server_port: usize) {
    let server_address:SocketAddr = format!("{server_addr}:{server_port}").to_socket_addrs()
        .expect("Unable to resolve domain")
        .next()
        .unwrap();

    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.connect(server_address).await.expect("Couldn't connect to server");

    // Create a new channel with a capacity of at most 32.
    let (tx, mut rx) = mpsc::channel(32); 
    let tx2 = tx.clone();
    
    let sock = Arc::new(Mutex::new(socket));
    let sock2 = sock.clone();
    // sends a message to the manager task to receive new udp messages
    let receive_handler = tokio::spawn(async move {
        println!("{}", "Starting the message receive handler".blue());
        loop {
            println!("{}", "Let's try to get new messages".blue());
            let (resp_tx, resp_rx) = oneshot::channel();

            let cmd = Command::Receive { responder: resp_tx };
            tx2.send(cmd).await.unwrap();
            println!("{}", "Sent to the channel".blue());

            match resp_rx.await {
                Ok(msg) => {
                    let msg = msg.unwrap();
                    println!("{}", "Message: ".blue());
                    println!("   {msg}");
                },
                Err(error) => {
                    println!("{}", "Receive timed out".blue());
                },
                
            };
        }
    });

    // reads input from the user and sends to the main thread
    let send_handler = tokio::spawn(async move {
        println!("{}", "Please, insert a message:".yellow());

        let mut sent: usize;
        loop {
            let (resp_tx, resp_rx) = oneshot::channel();
            let mut msg_line = String::new();
            let _ = std::io::stdin()
                .read_line(&mut msg_line)
                .unwrap();
            println!("{}", format!("Read {:?} bytes from user input.", msg_line.len()).yellow());

            let cmd = Command::Send { msg: msg_line, responder: resp_tx };
            tx.send(cmd).await.unwrap();
            println!("{}", "Sent to the channel".yellow());
            sent = resp_rx.await.unwrap().unwrap();
            if sent <= 0 {
                break;
            }
        } 
        
    });

    // receives messages from async functions and print
    let manager = tokio::spawn(async move {
        let sock2 = sock.clone();
        println!("{}", "Starting message manager... sending hello".green());
        let hello = "hello\n";
        let sent = sock2
            .lock()
            .await
            .send(hello.as_bytes())
            .await
            .unwrap();
        println!("{}", format!("{sent} bytes sent\n").green());
        while let Some(cmd) = rx.recv().await {
            println!("{}", "Received a new message".green());
            let cmd_type = match &cmd {
                Command::Receive { responder } => "Receive",
                Command::Send { msg, responder } => "Send"
            };
            println!("{}", format!("Received a new {:?} command", cmd_type).green());
            match cmd {
                Command::Receive { responder } => {
                    let _ = timeout(Duration::from_secs(5), async {
                        let mut buffer = vec![0;MAX_DATAGRAM_SIZE];
                        let rec_bytes = sock2
                            .lock()
                            .await
                            .recv(&mut buffer)
                            .await
                            .expect("No message received");
                        
                        
                        println!("{rec_bytes} {}", " bytes received: ".blue());
                        let buffer = &mut buffer[..rec_bytes];
                        let msg = String::from_utf8(buffer.to_vec()).unwrap();

                        responder.send(Ok(msg)).unwrap();
                    }).await;
                },
                Command::Send { msg, responder } => {
                    println!("{}", "Received the Send channel::message".green());
                    let sent = sock2
                        .lock()
                        .await
                        .send(msg.as_bytes())
                        .await
                        .unwrap();
                    println!("{sent} bytes sent\n");

                    responder.send(Ok(sent)).unwrap();
                }
            }
        }

        println!("{}", "Finished without receiving nothing".green());
    });


    receive_handler.await.unwrap();
    send_handler.await.unwrap();
    manager.await.unwrap();
}
