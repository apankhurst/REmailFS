mod setup;

use std::env;
use setup::Config;

//use fuse;

fn main() {
    // parse the command line arguments 
    let config = Config::new(env::args())
                    .expect("Unable to configure REmailFS");

    // parse command line arguments 
    // setup the filesystem
    // mount the file system
    
    /*let _result = fuse::mount(
        config.filesystem,
        &config.mountpoint,
        &[]
    );*/

}
