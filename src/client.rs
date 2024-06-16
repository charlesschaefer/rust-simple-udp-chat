use std::{io::{self, BufRead}, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, sync::{Arc, Mutex}, thread};

const MAX_DATAGRAM_SIZE: usize = 65535;


pub struct Client {
    socket: UdpSocket,
    server_address: SocketAddr,
}

impl Client {
    pub fn new(server_host: String, server_port: u16) -> Self {
        let server_address = SocketAddr::new(
            IpAddr::V4(
                Ipv4Addr::new(127, 0, 0, 1)
            ), 
            2222
        );


        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.connect(server_address).expect("Couldn't connect to server");

        Client { socket, server_address }
    }

    pub fn receive(this: Arc<Mutex<Self>>) {
        // tries to read something the user typed first
        // TODO: change this to work with threads
        let that = this.clone();
        let handle = thread::spawn(move || {
            loop {
                
                let mut buffer = vec![0;MAX_DATAGRAM_SIZE];
                let rec_bytes = this.lock().expect("Cant lock this variable").socket.recv(&mut buffer).expect("No message received");
                
                println!("{rec_bytes} bytes received: ");

                let buffer = &mut buffer[..rec_bytes];
                let msg = String::from_utf8(buffer.to_vec()).unwrap();
                
                println!("  {msg}");
            }
        });
        
        let handle2 = thread::spawn(move || {
            loop {
                println!("Please, insert a message:");
                let mut msg_line = String::new();
                io::stdin().lock().read_line(&mut msg_line).unwrap();
                that.lock().expect("Can't lock that variable").send(msg_line);
            }
        });

        handle.join();
        handle2.join();
    }

    pub fn send(&self, msg: String) {
        println!("Foi");
        //let msg = "Uma mensagem em uma garrafa".as_bytes();
        let sent = self.socket.send(msg.as_bytes()).unwrap();

        println!("Sent {sent} bytes");
    }
}
