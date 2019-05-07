extern crate libc;
extern crate time;
extern crate fuse;
extern crate imap;
extern crate native_tls;

use std::net::TcpStream;
use libc::c_int;
use imap::Session;
use fuse::Filesystem;
use fuse::Request;
use native_tls::{TlsConnector, TlsStream};

pub struct REmailFS {
    username: String,
    password: String,
    domain: String,
    port: u16,
    imap_session: Session<TlsStream<TcpStream>>,
}

impl REmailFS {
    pub fn new(uname: String, pword: String, domain: String, port: u16) -> Result<REmailFS, &'static str> {
        let tls = TlsConnector::builder().build().unwrap();
        println!("created tls");

        println!("username   = {}", uname);
        println!("password   = {}", pword);
        println!("domain     = {}", domain);
        println!("port       = {}", port);


        let client = match imap::connect((domain.as_str(), port), domain.as_str(), &tls){
            Ok(c) => c,
            Err(e) => {
                eprintln!("{:?}", e);
                return Err("failed to create IMAP client")
            },
        };
        println!("created client");
        
        let session = match client.login(uname.as_str(), pword.as_str()) {
            Ok(s) => s,
            Err(_) => return Err("failed to login"),
        };
        println!("created session");

        Ok(REmailFS { 
            username: uname, 
            password: pword, 
            domain: domain, 
            port: port,
            imap_session: session
        })
    }
}

impl Filesystem for REmailFS {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        println!("Entered init!");
        println!("Request: {:?}", _req);
        //self.imap_
        Ok(())
    }

    fn destroy(&mut self, _req: &Request) {
        println!("Entered destroy!");
        println!("Request: {:?}", _req);
    }
}
