use std::env;

const DEFAULT_DOMAIN: &str = "imap.google.com";
const DEFAULT_PORT: u16 = 993;

const usage: &str = "Usage: remailfs [OPTION]... MOUNT POINT
Mount an email account at the specified MOUNT POINT.

The following environment variables can be used to configure REmailFS:
REMAILFS_DOMAIN:    the IMAP server to connect to (default=imap.google.com) 
REMAILFS_PORT:      the port to connect to on the IMAP server (default=993)
REMAILFS_USERNAME:  username for the account 
REMAILFS_PASSWORD:  password for the account

*** IMPORTANT ***
Environment variables will override values in configuration files

Mandatory arguments to long options are mandatory for short options too.
-c, --config=FILE       use a config file to setup REmailFS
-h, --help              show this usage text
";

pub struct Config {
    username: String,
    password: String,
}
    
impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        // deal with configuration file here!
        
        let mut domain = match env::var("REMAILFS_DOMAIN") {
            Ok(d) => d,
            Err(e) => DEFAULT_DOMAIN.to_string(),
        };
        
        let mut port = match env::var("REMAILFS_PORT") {
            Ok(p) => p.parse::<u16>()
                        .expect("REMAILFS_PORT must be an unsigned 16-bit integer"),
            Err(e) => DEFAULT_PORT,
        };



        if !true {
            Ok(Config { username: "alex".to_string(), password: "password".to_string() } )
        } else {
            Err("Something went wrong!")
        }
    }
}

