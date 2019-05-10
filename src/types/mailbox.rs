use std::cmp::Ordering;
use std::vec::Vec;
use std::collections::{BTreeMap,BTreeSet};
use std::net::TcpStream;

pub type IMAPFlag<'a> = imap::types::Flag<'a>;
pub type IMAPMailbox = imap::types::Mailbox;


#[derive(Eq)]
pub struct Mailbox {
    name: String,
    children: BTreeMap<String, Mailbox>,
    info: Option<IMAPMailbox>,
}

impl Ord for Mailbox {
    fn cmp(&self, other: &Mailbox) -> Ordering {
        self.name.cmp(&other.name)
    }
}   

impl PartialOrd for Mailbox {
    fn partial_cmp(&self, other: &Mailbox) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Mailbox {
    fn eq(&self, other: &Mailbox) -> bool {
        self.name == other.name
    }
}

impl Mailbox {
    fn new(name: String) -> Mailbox {
        Mailbox {
            name: name,
            children: BTreeMap::new(),
            info: None,
        }
    }
   
    fn add_child(&mut self, name: String) {
        let mut split_path: Vec<&str> = name.split("/").collect();
        let my_size = match self.name == "/" {
            true => 0,
            false => self.name.split("/").count(),
        };

        if split_path.len() - 1 == my_size {
            self.children.insert(name.clone(), Mailbox::new(name));
        } else {
            split_path.truncate(my_size+1);
            let next_dir = split_path.join("/");
            let mut next_mb = match self.children.get_mut(&next_dir) {
                Some(mb) => mb,
                None => {
                    self.children.insert(next_dir.clone(), Mailbox::new(next_dir.clone()));
                    self.children.get_mut(&next_dir).unwrap()
                },
            };
            next_mb.add_child(name);
        }
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
        println!("{:\t<0$}mailbox: {}", tabs, self.name);
        for (_, mb) in self.children.iter() {
            mb.print(tabs + 1);
        }
    }
}
