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
use imap::types::{Uid, Name, Fetch};
use libc::{ENOENT, ENOSYS};
use time::Timespec;

mod error;

pub type IMAPFlag<'a> = imap::types::Flag<'a>;
pub type IMAPMailbox = imap::types::Mailbox;
pub type IMAPSession = Session<TlsStream<TcpStream>>;
pub type IMAPFetch = imap::types::Fetch;

enum EmailObject<'a> {
    E(&'a Email),
    M(&'a Mailbox),
}

pub struct Email {
    abs_path: String,
    contents: Option<Fetch>,
}

impl Email {
    fn new(abs_path: &str) -> Email {
        Email {
            abs_path: abs_path.to_string(),
            contents: None,
        }
    }

    fn set_contents(&mut self, fetch: Fetch) {
        self.contents = Some(fetch);
    }
}

pub struct Mailbox {
    abs_path: String,
    info: Option<IMAPMailbox>,
    contents: BTreeSet<u64>,
}

impl Mailbox {
    fn new(abs_path: &str) -> Mailbox {
        Mailbox {
            abs_path: abs_path.to_string(),
            info: None,
            contents: BTreeSet::new(),
        }
    }

    fn add_content(&mut self, inode: u64) {
        self.contents.insert(inode);
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
    emails: BTreeMap<u64, Email>,
    mailboxes: BTreeMap<u64, Mailbox>,
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

        Ok(REmailFS { 
            username: uname, 
            password: pword, 
            domain: domain, 
            port: port,
            next_inode: 2,
            imap_session: session,
            inodes: BTreeMap::new(),
            emails: BTreeMap::new(),
            mailboxes: BTreeMap::new(),
            attributes: BTreeMap::new(),
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

        let root_mailbox = Mailbox::new("/");

        self.inodes.insert("/".to_string(), 1);
        self.mailboxes.insert(1, root_mailbox);
        self.attributes.insert(1, root_attrs);

        let mut root_mailbox = self.mailboxes.get_mut(&1)
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
            let inode = self.next_inode;
            let mut uids = None; 
            let mut abs_path = mb.to_string();
            let mut mailbox = Mailbox::new(&abs_path);
           
            self.next_inode += 1;

            mailbox.info = match self.imap_session.select(*mb) {
                Ok(mb) => Some(mb),
                Err(_) => None,
            };

            if mailbox.info.is_some() {
                uids = match self.imap_session.uid_search("1:*") {
                    Ok(u) => Some(u),
                    Err(_) => continue
                };
            }

            let mailbox_attrs =  FileAttr {
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
            self.mailboxes.insert(inode, mailbox);
            self.attributes.insert(inode, mailbox_attrs);

            if uids.is_some() {
                let mut parent = self.mailboxes.get_mut(&inode).unwrap();

                for uid in uids.unwrap() {
                    let mut path = abs_path.clone();
                    let u_inode = self.next_inode;

                    path.push('/');
                    path.push_str(uid.to_string().as_str());
                    self.next_inode += 1;
    
                    let email = Email::new(&path);

                    let email_attrs =  FileAttr {
                        ino: u_inode,
                        size: 4096,
                        blocks: 1,
                        atime: now,
                        mtime: now,
                        ctime: Timespec::new(0,0),
                        crtime: Timespec::new(0,0),
                        kind: FileType::RegularFile,
                        perm: 0o444,
                        nlink: 1,
                        uid: _req.uid(),
                        gid: _req.gid(),
                        rdev: 0,
                        flags: 0,
                    };

                    self.inodes.insert(path.clone(), u_inode);
                    self.emails.insert(u_inode, email);
                    self.attributes.insert(u_inode, email_attrs);

                    parent.add_content(u_inode);

                    println!("{}", path); 
                }
            }

            let mut split_path: Vec<&str> = abs_path.rsplitn(2, "/")
                .collect();
            let mut p_inode = &1;

            if split_path.len() > 1 { 
                p_inode = self.inodes.get(split_path[1]).unwrap();
            }

            let mut parent = self.mailboxes.get_mut(p_inode).unwrap();
            parent.add_content(inode);
        }
/*
        let mb = self.imap_session.examine("INBOX").unwrap();
        let messages = self.imap_session.fetch("1:333", "RFC822").unwrap();
        for message in messages.iter() {
            let body = message.body().expect("message did not have a body!");
            let body = std::str::from_utf8(body)
                .expect("message was not valid utf-8")
                .to_string();

            println!("{}", body);

        }*/   
       /* 
        let message = if let Some(m) = messages.iter().next() {
            m
        } else {
            return Ok(());
        };

        // extract the message's body
        let body = message.body().expect("message did not have a body!");
        let body = std::str::from_utf8(body)
            .expect("message was not valid utf-8")
            .to_string();

        println!("{}", body);
*/        /*let emails = self.imap_session.fetch("1", "RFC822").unwrap();

        for email in emails.iter() {
            println!("-------------------------");
            let envelope = match email.envelope() {
                Some(e) => e,
                None => continue
            };
            let subject = match envelope.subject {
                Some(s) => s,
                None => continue
            };
 
            println!("{}", std::str::from_utf8(email.body().unwrap()).unwrap().to_string()); 
        }*/

        println!("GOT EMAILS");

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
            println!("ENOENT in getattr");
            reply.error(ENOENT);
        }
    }

    fn readdir(&mut self, _req: &Request, _ino: u64, _fh: u64, _offset: i64, mut reply: ReplyDirectory) {
        println!("readdir(ino = {}, fh = {})", _ino, _fh);

        let mailbox = self.mailboxes.get(&_ino);
        
        let mut mailbox = if mailbox.is_none() {
            println!("ENOENT in readdir");
            reply.error(ENOENT);
            return;
        } else {
            mailbox.unwrap()

        };

        if _offset == 0 {
            let mut count = 2;
            let mut info = &mailbox.info;
            let contents = &mailbox.contents;
            
            reply.add(1, 0, FileType::Directory, ".");
            reply.add(1, 1, FileType::Directory, "..");

            for inode in contents.iter() {
                let base: EmailObject = if self.mailboxes.get(inode).is_some() {
                    EmailObject::M(self.mailboxes.get(inode).unwrap())
                } else {
                    EmailObject::E(self.emails.get(inode).unwrap())
                };
                
                let rel_path = match base {
                    EmailObject::E(e) => e.abs_path.clone(),
                    EmailObject::M(m) => m.abs_path.clone(),
                };

                let rel_path = rel_path
                    .rsplitn(2, "/")
                    .next()
                    .unwrap();
                
                let f_type = self.attributes.get(inode)
                    .unwrap()
                    .kind;

                reply.add(*inode, 2+count as i64, f_type, rel_path); 
                count += 1;

            }

            reply.ok();
        }
    }

    fn lookup(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEntry) {
        println!("lookup(parent = {}, name = {:#?})", _parent, _name);

        let _name = _name.to_str()
            .unwrap();

        let mut abs_path = if _parent != 1 {
            self.mailboxes.get(&_parent)
                .unwrap()
                .abs_path
                .clone()
        } else {
            "".to_string()
        };
        
        if _parent != 1 { abs_path.push('/') }; 

        abs_path.push_str(_name);

        let inode = match self.inodes.get(&abs_path) {
            Some(i) => i,
            None => {
                println!("ENOENT in lookup1");
                reply.error(ENOENT);
                return;
            }
        };

        let attrs = self.attributes.get(inode);

        if let Some(a) = attrs {
            let ttl = Timespec::new(1, 0);
            reply.entry(&ttl, a, 1);
        } else {
            println!("ENOENT in lookup2");
            reply.error(ENOENT);
        }
    }
}
