use std::{env, process, sync::{Arc, Mutex}};
use server::Server;
use client::Client;

mod server;
mod client;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("udp_chat_server {{type}}, where {{type}} can be 'server' or 'client'");
        process::exit(1);
    }

    let module_type = &args[1];

    if module_type == "server" {
        /* let mut server = Server::new("localhost".to_string(), 2222);
        server.receive(); */
        Server::start("localhost".to_string(), 2222).await;
    } else if module_type == "client" {
        let mut client = Client::new("localhost".to_string(), 2222);
        Client::receive(Arc::new(Mutex::new(client)));
    }
}


