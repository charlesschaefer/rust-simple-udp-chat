use std::{fmt::format, io::Read, net::{SocketAddr, UdpSocket}};
use tokio::sync::{mpsc, mpsc::{Sender, Receiver}};


const MAX_DATAGRAM_SIZE: usize = 65535;

enum Command {
    Send {
        msg: String,
        source: SocketAddr,
    }
}

pub struct Client {
    id: String,
    address: SocketAddr
}

impl Client {
    pub fn new(address: SocketAddr) -> Self {
        let id = Client::get_id_from_address(address);
        Client { address, id }
    }

    pub fn get_id_from_address(address: SocketAddr) -> String {
        let (host, port) = (
            address.ip().to_string(),
            address.port().to_string()
        );
        format!("{host}:{port}")
    }
}


pub struct Server {
    clients: Vec<Client>,
    socket: UdpSocket,
    channel: (Sender::<Command>, Receiver::<Command>),
}

impl Server {
    pub fn new(host: String, port: u16) -> Self {
        let socket = UdpSocket::bind((host, port)).unwrap();

        let clients: Vec<Client> = vec![];

        let channel = mpsc::channel(32);

        Server { clients, socket, channel }
    }

    pub async fn start(&mut self) {
        let that = self.clone();
        let manager_task = tokio::spawn(async move {
            // Start receiving messages from the channel
            while let Some(cmd) = self.channel.1.recv().await {
                match cmd {
                    Command::Send { msg, source } => {
                        self.send_received_message(msg, source);
                    },
                }
            }
        });

        let receive_task = tokio::spawn(async move {
            that.receive();
        });

        receive_task.await.unwrap();
        manager_task.await.unwrap();
    }

    pub async fn receive(&mut self) {
        loop {
            let mut buffer = vec![0;MAX_DATAGRAM_SIZE];
            let (rec_bytes, source) = self
                .socket
                .recv_from(&mut buffer)
                .expect("No message received"); 
            // let rec_bytes = self.socket.recv(&mut buffer).expect("No message received");
            println!("{rec_bytes} bytes received");

            let buffer = &mut buffer[..rec_bytes-1];
            let msg = String::from_utf8(buffer.to_vec()).unwrap();

            self._add_client(source);

            let envelop = Command::Send { msg, source };
            self.channel.0.send(envelop).await.unwrap();

            // tokio::spawn(async move {
            //     self.send_received_message(msg, source);
            // });
        }
    }

    pub fn send_received_message(&self, msg: String, source: SocketAddr) {
        let (host, port) = (
            source.ip().to_string(),
            source.port().to_string()
        );
        let new_message = format!("from {host}:{port} => \"{msg}\"");
        println!("Message being sent:\n  {new_message}");


        for client in &self.clients {
            if client.id != Client::get_id_from_address(source) {
                let _sent = self.socket.send_to(new_message.as_bytes(), client.address).unwrap();

                println!("{_sent} bytes sent");
            }
        }
    }

    fn _add_client(&mut self, client_addr: SocketAddr) {
        let client = Client::new(client_addr);
        for cli in &self.clients {
            if cli.id == client.id {
                return;
            }
        }
        let id = &client.id;
        println!("Adding client {id}");
        self.clients.push(client);
    }

}