extern crate libc;
extern crate time;
extern crate fuse;

use std::ffi::OsStr;
use std::os::raw::c_int;
use std::path::Path;
use libc::{ENOENT, ENOSYS};
use time::{Timespec};
use fuse::{Filesystem, Request, ReplyAttr, ReplyDirectory, ReplyEntry, ReplyData, FileAttr, FileType};

pub struct REmailFS {
    pub username: String,
    pub password: String,
    pub domain: String,
    pub port: u16,
}

impl REmailFS {
    pub fn new(username: String, password: String, domain: String, port: u16) -> REmailFS {
        REmailFS { username: username, password: password, domain: domain, port: port }
    }
}

impl Filesystem for REmailFS {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        println!("Entered init!");
        println!("Request: {:?}", _req);
        Ok(())
    }

    fn destroy(&mut self, _req: &Request) {
        println!("Entered destroy!");
        println!("Request: {:?}", _req);
    }

    fn lookup(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEntry) {
        let name = _name.to_str().unwrap();
        println!("lookup(parent={}, name={})", _parent, name);
        reply.error(ENOSYS);
    }

    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {
        println!("forget(ino={}, nlookup={})", _ino, _nlookup);
    }

    fn getattr(&mut self, _req: &Request, _ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", _ino);   
        reply.error(ENOSYS);
    }

    fn setattr(&mut self, _req: &Request, _ino: u64, _mode: Option<u32>, _uid: Option<u32>, 
               _gid: Option<u32>, _size: Option<u64>, _atime: Option<Timespec>, 
               _mtime: Option<Timespec>, _fh: Option<u64>, _crtime: Option<Timespec>, 
               _chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, _flags: Option<u32>, 
               reply: ReplyAttr) {
        
        println!("setattr={}", _ino);
        reply.error(ENOSYS);
    }

    fn readlink(&mut self, _req: &Request, _ino: u64, reply: ReplyData) {
        println!("setattr={}", _ino);
        reply.error(ENOSYS);
    }

    fn mknod(
        &mut self, 
        _req: &Request, 
        _parent: u64, 
        _name: &OsStr, 
        _mode: u32, 
        _rdev: u32, 
        reply: ReplyEntry
        ) {
        println!("mknod");
        reply.error(ENOSYS);
    }

    fn mkdir(
        &mut self, 
        _req: &Request, 
        _parent: u64, 
        _name: &OsStr, 
        _mode: u32, 
        reply: ReplyEntry
        ) {
        println!("mkdir(parent={}, name={}, mode={})", _parent, _name.to_str().unwrap(), _mode);
        reply.error(ENOSYS);
    }

    fn readdir(&mut self, _req: &Request, _ino: u64, _fh: u64,
               _offset: i64, mut reply: ReplyDirectory) {
        println!("readdir(ino={}, fh={}, offset={})", _ino, _fh, _offset);
        if _ino == 1 {
            if _offset == 0 {
                reply.add(1, 0, FileType::Directory, &Path::new("."));
                reply.add(1, 1, FileType::Directory, &Path::new(".."));
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }
}
