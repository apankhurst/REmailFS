use crate::IMAPSession;
use imap::types::*;

pub fn get_mailboxes(imap_session: &mut IMAPSession) -> Option<ZeroCopy<Vec<Name>>> {
    let mailboxes = imap_session.list(Some(""), Some("*"));
    if !mailboxes.is_err() {
        return Some(mailboxes.unwrap())
    }

    return None
}

pub fn select_mailbox(imap_session: &mut IMAPSession, name: &str) -> Option<Mailbox> {
    match imap_session.select(name) {
        Ok(mb) => Some(mb), 
        Err(_) => {
          println!("error fetching mailbox {}", name);
          None
        },
    }
}
