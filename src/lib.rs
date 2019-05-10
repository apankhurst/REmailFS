extern crate libc;
extern crate time;
extern crate fuse;
extern crate imap;
extern crate native_tls;

use std::cmp::Ord;
use std::ffi::OsStr;
use std::cmp::Ordering;
use std::vec::Vec;
use std::collections::{BTreeMap,BTreeSet};
use std::net::TcpStream;
use libc::c_int;
use imap::Session;
use fuse::Filesystem;
use fuse::*;
use native_tls::{TlsConnector, TlsStream};
use imap::types::{Uid, Name};
use libc::{ENOENT, ENOSYS};
use time::Timespec;

mod error;

pub type IMAPFlag<'a> = imap::types::Flag<'a>;
pub type IMAPMailbox = imap::types::Mailbox;
pub type IMAPSession = Session<TlsStream<TcpStream>>;


pub struct Mailbox {
    abs_path: String,
    info: Option<IMAPMailbox>,
    //emails: BTreeMap<u64, Email>,
    children: BTreeSet<u64>,
}

impl Mailbox {
    fn new(abs_path: &str) -> Mailbox {
        Mailbox {
            abs_path: abs_path.to_string(),
            children: BTreeSet::new(),
            info: None,
        }
    }
   
    fn add_child(&mut self, inode: u64) {
        self.children.insert(inode);
    }

    fn set_info(&mut self, info: IMAPMailbox) {
        self.info = Some(info);
    }

    fn has_info(&self) -> bool {
        self.info.is_some()
    }

    fn flags(&self) -> Option<Vec<IMAPFlag>> {
        if let Some(i) = &self.info {
            Some(i.flags.clone())
        } else {
            None
        }
    }
    
    fn permanent_flags(&self) -> Option<Vec<IMAPFlag>> {
        if let Some(i) = &self.info {
            Some(i.permanent_flags.clone())
        } else {
            None
        }
    }

    fn exists(&self) -> u32 {
        if let Some(i) = &self.info {
            i.exists
        } else {
            0
        }
    }

    fn recent(&self) -> u32 {
        if let Some(i) = &self.info {
            i.recent
        } else {
            0
        }
    }

    fn unseen(&self) -> Option<u32> {
        if let Some(i) = &self.info {
            i.unseen
        } else {
            None
        }
    }

    fn uid_next(&self) -> Option<Uid> {
        if let Some(i) = &self.info {
            i.uid_next
        } else {
            None
        }
    }

    fn uid_validity(&self) -> Option<Uid> {
        if let Some(i) = &self.info {
            i.uid_validity
        } else {
            None
        }
    }

    fn print(&self, tabs: usize) {
        println!("{:\t<0$}mailbox: {}", tabs, self.abs_path);
    }
}

pub struct REmailFS {
    username: String,
    password: String,
    domain: String,
    port: u16,
    next_inode: u64,
    imap_session: IMAPSession,
    inodes: BTreeMap<String, u64>,
    contents: BTreeMap<u64, Mailbox>,
    attributes: BTreeMap<u64, FileAttr>,
}

impl REmailFS {
    pub fn new(uname: String, pword: String, domain: String, port: u16) -> Result<REmailFS, &'static str> {
        let tls = TlsConnector::builder().build().unwrap();
        println!("created tls");
        
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
            Err(e) => {
                eprintln!("{:?}", e);
                return Err("failed to login");
            },
        };
        println!("created session");

        let inodes = BTreeMap::new();
        let contents = BTreeMap::new();
        let attributes = BTreeMap::new();

        Ok(REmailFS { 
            username: uname, 
            password: pword, 
            domain: domain, 
            port: port,
            next_inode: 2,
            imap_session: session,
            inodes: inodes,
            contents: contents,
            attributes: attributes,
        })
    }
}

impl Filesystem for REmailFS {
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        println!("Entered init!");
        
        let now = time::now().to_timespec();;

        let mut root_attrs = FileAttr {
            ino: 1,
            size: 4096,
            blocks: 1,
            atime: now,
            mtime: now,
            ctime: Timespec::new(0,0),
            crtime: Timespec::new(0,0),
            kind: FileType::Directory,
            perm: 0o444,
            nlink: 1,
            uid: _req.uid(),
            gid: _req.gid(),
            rdev: 0,
            flags: 0,
        };

        let root_contents = Mailbox::new("/");

        self.inodes.insert("/".to_string(), 1);
        self.contents.insert(1, root_contents);
        self.attributes.insert(1, root_attrs);
       
        let mut root_contents = self.contents.get_mut(&1)
                                    .unwrap();

        let mut all_boxes = match self.imap_session.list(Some(""), Some("*")) {
            Ok(ab) => ab,
            Err(_) => return Err(-1)
        };

        let mut all_boxes: Vec<&str> = all_boxes.iter()
                                        .map(|n| n.name().clone())
                                        .collect();

        all_boxes.sort_unstable();

        for mb in all_boxes.iter() {
            println!("adding {}", *mb);
            let mut abs_path = mb.to_string();
             
            let inode = self.next_inode; 
            let contents = Mailbox::new(&abs_path);
            let attrs =  FileAttr {
                ino: inode,
                size: 4096,
                blocks: 1,
                atime: now,
                mtime: now,
                ctime: Timespec::new(0,0),
                crtime: Timespec::new(0,0),
                kind: FileType::Directory,
                perm: 0o444,
                nlink: 1,
                uid: _req.uid(),
                gid: _req.gid(),
                rdev: 0,
                flags: 0,
            };

            self.inodes.insert(abs_path.clone(), inode);
            self.contents.insert(inode, contents);
            self.attributes.insert(inode, attrs);

            let mut split_path: Vec<&str> = abs_path.rsplitn(2, "/")
                                            .collect();
            let mut p_inode = &1;

            if split_path.len() > 1 { 
                p_inode = self.inodes.get(split_path[1]).unwrap();
            }
            
            let mut parent = self.contents.get_mut(p_inode).unwrap();
            parent.add_child(inode);

            self.next_inode += 1;
        }
        
        Ok(())
    }

    fn destroy(&mut self, _req: &Request) {
        println!("Entered destroy!");

        let _ = self.imap_session.logout();
    }

    fn getattr(&mut self, _req: &Request, _ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", _ino);
        let attrs = self.attributes.get(&_ino);
        if let Some(a) = attrs {
            let ttl = Timespec::new(1, 0);
            reply.attr(&ttl, a);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(&mut self, _req: &Request, _ino: u64, _fh: u64, _offset: i64, mut reply: ReplyDirectory) {
        println!("readdir(ino = {}, fh = {})", _ino, _fh);
        let mailbox = self.contents.get(&_ino);
        
        if let Some(mb) = mailbox {
            if _offset == 0 {
                reply.add(1, 0, FileType::Directory, ".");
                reply.add(1, 1, FileType::Directory, "..");

                for (count, inode) in mb.children.iter().enumerate() {
                    let rel_path = self.contents.get(inode)
                                    .unwrap()
                                    .abs_path
                                    .rsplitn(2, "/")
                                    .next()
                                    .unwrap();
                    let f_type = self.attributes.get(inode)
                                    .unwrap()
                                    .kind;
                    
                    reply.add(*inode, 2+count as i64, f_type, rel_path); 
                }
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }

    fn lookup(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEntry) {
        println!("lookup(name = {:#?})", _name);
        
        let _name = _name.to_str()
                        .unwrap();
        let mut abs_path = match _parent {
                            1 => { self.contents.get(&_parent)
                                    .unwrap()
                                    .abs_path
                                    .clone()
                            },
                                _ => "".to_string(),
                        };
        
        if _parent != 1 { abs_path.push('/') }; 
        abs_path.push_str(_name);
        
        let inode = match self.inodes.get(&abs_path) {
            Some(i) => i,
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        let attrs = self.attributes.get(inode);
        
        if let Some(a) = attrs {
            let ttl = Timespec::new(1, 0);
            reply.entry(&ttl, a, 1);
        } else {
            reply.error(ENOENT);
        }
    }
}
