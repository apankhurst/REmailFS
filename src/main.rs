mod setup;

use std::env;
use setup::Config;

//use fuse;

fn main() {
    // parse the command line arguments 
    // the config object should contain the filesystem
    // the mountpoint and 
    
    let config = Config::new(env::args());

    let config = match config {
        Some(c) => c,
        None => {
            println!("failed to configure...");
            return
        },
    };
    
    // parse command line arguments 
    // setup the filesystem
    // mount the file system

    let result = fuse::mount(
        config.filesystem,
        &config.mountpoint,
        &[]
    );

    if result.is_err() {
        eprintln!("{:?}", result);
    }
}
