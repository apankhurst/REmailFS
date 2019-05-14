extern crate libc;
extern crate time;
extern crate fuse;
extern crate imap;
extern crate mailparse;
extern crate imap_proto;
extern crate native_tls;

use std::borrow::BorrowMut;
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
use imap_proto::types::Address;
use imap::types::{Uid, Name, Fetch};
use libc::{ENOENT, ENOSYS};
use time::Timespec;
use time::strptime;

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
    contents: Option<String>,
    subject: Option<String>,
    from: Option<String>,
    date: Option<String>,
}

impl Email {
    fn new(abs_path: &str) -> Email {
        Email {
            abs_path: abs_path.to_string(),
            contents: None,
            subject: None,
            from: None,
            date: None,
        }
    }

    fn set_contents(&mut self, contents: String) {
        self.contents = Some(contents);
    }

    fn set_subject(&mut self, subject: String) {
        self.subject = Some(subject);
    }

    fn set_from(&mut self, from: String) {
        self.from = Some(from);
    }

    fn set_date(&mut self, date: String) {
        self.date = Some(date)
    }

    fn contents_as_bytes(&self) -> Vec<u8> {
        if let Some(c) = &self.contents {
            c.clone().as_bytes().to_vec()
        } else {
            "".as_bytes().to_vec()
        }
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
            let mut abs_path = mb.to_string();
            let mut mailbox = Mailbox::new(&abs_path);

            self.next_inode += 1;

            mailbox.info = match self.imap_session.examine(*mb) {
                Ok(mb) => Some(mb),
                Err(_) => None
            };

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

            let mut parent = self.mailboxes.get_mut(&inode).unwrap();
            let uids = match self.imap_session.uid_search("1:*") {
                Ok(u) => Some(u),
                Err(_) => None,
            };

            if uids.is_some() {
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

                //println!("{}", path); 
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
        println!("REmailFS is ready to use!");
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
                    println!(">>> READDIR ON EMAIL <<<");
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

            println!("CONTENT COUNT = {}", count);

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

    fn read(&mut self, _req: &Request, _ino: u64, _fh: u64, _offset: i64, _size: u32, reply: ReplyData) {
        println!("read(_ino = {})", _ino);
        let mut email = self.emails.get_mut(&_ino);

        if email.is_none() {
            reply.error(ENOENT);
            return;
        }

        let mut email = email.unwrap();

        if email.contents.is_none() {
            let split_path: Vec<&str> = email.abs_path.rsplitn(2, "/").collect();    

            let uid = split_path[0];
            let parent = split_path[1];

            self.imap_session.examine(parent);

            let contents = self.imap_session.uid_fetch(uid, "RFC822");

            if contents.is_err() {
                reply.error(ENOENT);
                return;
            } 

            let mut data = "".to_string();

            let fetch: &Fetch = &contents.unwrap()[0];
            let body = fetch.body();;
            let mut body_str = "".to_string();
                
            if body.is_some() {
                let new_body_str = std::str::from_utf8(body.unwrap());
                if new_body_str.is_ok() {
                    body_str.push_str(&new_body_str.unwrap().to_string());
                }
            }

            data.push_str(&body_str);
            email.contents = Some(data);
        }

        let mut email = self.emails.get_mut(&_ino).unwrap();
        let contents = email.contents.clone().unwrap();

        println!(">>> EMAIL = {}", email.contents.clone().unwrap());

        let mut reply_text = "".to_string();
        let mut add_key_val = |k: &str, v: &str| {
            reply_text.push_str(k.clone());
            reply_text.push_str(": ");
            reply_text.push_str(v.clone());
            reply_text.push('\n');
        };

        let parsed = mailparse::parse_mail(contents.as_bytes()).unwrap();

        for header in parsed.headers {
            let key = header.get_key().unwrap();
            let val = header.get_value().unwrap();

            match key.as_str() {
                "Subject" => {
                    let mut new_path = "".to_string(); 
                    let split_path: Vec<&str> = email.abs_path.rsplitn(2, "/").collect();
                    new_path.push_str(split_path[1]);
                    new_path.push('/');
                    new_path.push_str(val.as_str());

                    email.abs_path = new_path.to_string();
                    self.inodes.insert(new_path, _ino);

                    add_key_val(key.as_str(), val.as_str())
                }
                "From"      => add_key_val(key.as_str(), val.as_str()), 
                "Date"      => {
                    // Mon, 15 Apr 2019 17:49:15 -0500 (CDT)   
                    let tm = strptime(val.as_str(), "%a, %d %b %Y %H:%M:%S");
                    if tm.is_ok() {
                        let tm = tm.unwrap().to_timespec();
                        let mut attr = self.attributes.get_mut(&_ino).unwrap();
                        attr.atime = tm;
                        attr.mtime = tm;
                        attr.ctime = tm;
                        attr.crtime = tm;
                        println!(">>> FORMATTED TIME");
                    } else {
                        println!(">>> Unable to format time");
                    }
                    add_key_val(key.as_str(), val.as_str())
                },
                _ => (),
            }
        }


        println!("{}", reply_text);

        reply.data(reply_text.as_bytes());
    }
}
