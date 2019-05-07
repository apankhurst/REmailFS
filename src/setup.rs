extern crate getopts;

use std::env;
use getopts::Options;
use remailfs::REmailFS;

const DEFAULT_DOMAIN: &str = "imap.gmail.com";
const DEFAULT_PORT: u16 = 993;

const USAGE: &str = "Usage: remailfs [OPTION]... MOUNT POINT
Mount an email account at the specified MOUNT POINT.

The following environment variables can be used to configure REmailFS:
REMAILFS_DOMAIN:    the IMAP server to connect to (default=imap.google.com) 
REMAILFS_PORT:      the port to connect to on the IMAP server (default=993)
REMAILFS_USERNAME:  username for the account 
REMAILFS_PASSWORD:  password for the account

*** IMPORTANT ***
Configuration value location priority:
1) Command line
2) Environment variables
3) Configuration file

Mandatory arguments to long options are mandatory for short options too.
-u, --uname=USERNAME    
-p, --pword=PASSWORD
-d, --domain=DOMAIN
-t, --port=PORT
-h, --help              show usage text
";

fn print_usage() {
    println!("{}", USAGE);
}

fn setup_opts(opt: &mut Options) {
    opt.optflag("h", "help", "show usage text");
    opt.optopt("u", "uname", "the username", "USERNAME");
    opt.optopt("p", "pword", "the password", "PASSWORD");
    opt.optopt("d", "domain", "the domain of the server", "DOMAIN");
    opt.optopt("t", "port", "the port to connect to", "PORT");
}

pub struct Config {
    pub filesystem: REmailFS,
    pub mountpoint: String,
}
    
impl Config {
    pub fn new(args: env::Args) -> Option<Config> {
        let args: Vec<String> = args.collect();
       
        if args.len() < 2 {
            eprintln!("too few arguments");
            return None;
        }

        let mut opts = Options::new();
        setup_opts(&mut opts);

        let matches = match opts.parse(&args[1..]) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{:?}", e);
                return None
            }
        };

        if matches.opt_present("h") || matches.free.is_empty() {
            print_usage();
            return None;
        }

        let mountpoint = matches.free[0].clone();
        
        let find_var = |cmd_ln, env_var, default| {
            match env::var(env_var) {
                Ok(s) => Ok(s),
                Err(_) => {
                    match matches.opt_str(cmd_ln) {
                        Some(s) => Ok(s),
                        None => {
                            match default {
                                Some(v) => Ok(v),
                                None => Err("unable to necessary environment variable"), 
                            }
                        },
                    }
                },
            }
        };

        let username = find_var("u", "REMAILFS_USERNAME", None)
                        .expect("no username");
        
        let password = find_var("p", "REMAILFS_PASSWORD", None)
                        .expect("no password");

        let domain = find_var("d", "REMAILFS_DOMAIN", Some(DEFAULT_DOMAIN.to_string()))
                        .unwrap();

        let port = find_var("t", "REMAILFS_PORT", Some(DEFAULT_PORT.to_string()))
                        .unwrap()
                        .parse::<u16>()
                        .unwrap();
        
        println!("username   = {}", username);
        println!("password   = {}", password);
        println!("domain     = {}", domain);
        println!("port       = {}", port);
        println!("mountpoint = {}", mountpoint);

        let fs = match REmailFS::new(
            username,
            password,
            domain,
            port
        ) {
            Ok(fs) => fs,
            Err(e) => return None,
        };

        println!("created filesystem");

        Some(Config { filesystem: fs, mountpoint: mountpoint })

    }
}

