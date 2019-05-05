extern crate libc;
extern crate time;
extern crate fuse;

use std::ffi::OsStr;
use std::os::raw::c_int;
use std::path::Path;
use std::env;
use std::collections::BTreeMap;
use libc::{ENOENT, ENOSYS};
use time::{Timespec};
use fuse::{Filesystem, Request, ReplyAttr, ReplyDirectory, ReplyEntry, ReplyData, FileAttr, FileType};

pub struct Config {
    pub mountpoint: String,
    pub filesystem: REmailFS,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let mountpoint = match args.next() {
            Some(mntpt) => mntpt,
            None => return Err("Too few arguments"), 
        };
        
        let filesystem = REmailFS::new();

        Ok(Config { mountpoint, filesystem })
    }
}

pub struct REmailFS {
    inodes: BTreeMap<String, u64>,
    attrs: BTreeMap<u64, FileAttr>,
}

const UNIX_EPOCH: Timespec = Timespec { sec: 0, nsec: 0};
const ROOT_ATTRS: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH,
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0
};

impl REmailFS {
    pub fn new() -> REmailFS {
        let mut inodes = BTreeMap::new();
        let mut attrs = BTreeMap::new();

        inodes.insert("/".to_string(), 1);
        attrs.insert(1, ROOT_ATTRS);

        REmailFS { inodes: inodes, attrs: attrs }
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
        if self.attrs.contains_key(&_ino) {
            let ttl = Timespec::new(1, 0);
            let attr: &mut FileAttr = self.attrs.get_mut(&_ino).unwrap();
            reply.attr(&ttl, &attr);
        } else {
            reply.error(ENOENT);
        }
    }

    fn setattr(&mut self, _req: &Request, _ino: u64, _mode: Option<u32>, _uid: Option<u32>, 
               _gid: Option<u32>, _size: Option<u64>, _atime: Option<Timespec>, 
               _mtime: Option<Timespec>, _fh: Option<u64>, _crtime: Option<Timespec>, 
               _chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, _flags: Option<u32>, 
               reply: ReplyAttr) {
        
        println!("setattr={}", _ino);
        if self.attrs.contains_key(&_ino) {
            let attr: &mut FileAttr = self.attrs.get_mut(&_ino).unwrap();
            let ttl = Timespec::new(0, 0);
    
            if let Some(uid) = _uid {
                attr.uid = uid;
            }

            if let Some(gid) = _gid {
                attr.gid = gid;
            }

            if let Some(size) = _size {
                attr.size = size;
            }

            if let Some(atime) = _atime {
                attr.atime = atime;
            }

            if let Some(mtime) = _mtime {
                attr.mtime = mtime;
            }

            //if let Some(fh) = _fh {
            //    attr.fh = fh;
            //}

            if let Some(crtime) = _crtime {
                attr.crtime = crtime;
            }

            if let Some(flags) = _flags {
                attr.flags = flags;
            }

            reply.attr(&ttl, &attr);
        } else {
            reply.error(ENOENT);
        }
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
