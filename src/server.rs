use std::{fmt::format, io::Read, net::{SocketAddr, UdpSocket}};

const MAX_DATAGRAM_SIZE: usize = 65535;
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
        format!("{:?}:{:?}", 
            address.ip().to_string(),
            address.port().to_string()
        )
    }
}
pub struct Server {
    clients: Vec<Client>,
    socket: UdpSocket,
}

impl Server {
    pub fn new(host: String, port: u16) -> Self {
        let socket = UdpSocket::bind((host, port)).unwrap();

        let clients: Vec<Client> = vec![];

        Server { clients, socket }
    }

    pub fn receive(&mut self) {
        loop {
            let mut buffer = vec![0;MAX_DATAGRAM_SIZE];
             let (rec_bytes, source) = self
                .socket
                .recv_from(&mut buffer)
                .expect("No message received"); 
            // let rec_bytes = self.socket.recv(&mut buffer).expect("No message received");
            
            println!("{rec_bytes} of a Message received: ");

            let buffer = &mut buffer[..rec_bytes];
            let msg = String::from_utf8(buffer.to_vec()).unwrap();

            self._add_client(source);

            self.send_received_message(msg, source);
        }
    }

    pub fn send_received_message(&self, msg: String, source: SocketAddr) {
        let new_message = format!(
            "Message received from {:?}:{:?} => \"{:?}\"",
            source.ip().to_string(),
            source.port().to_string(),
            msg
        );
        println!("Message being sent: {new_message}");


        for client in &self.clients {
            if client.id != Client::get_id_from_address(source) {
                let _sent = self.socket.send_to(new_message.as_bytes(), client.address);
                println!("Bytes sent to clients: {:?} bytes", _sent.unwrap());
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
        println!("Adding the client {:?}", client.id);
        self.clients.push(client);
    }

}