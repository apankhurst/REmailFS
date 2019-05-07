use crate::IMAPSession;
use imap::types::{ZeroCopy, Name};

pub fn get_mailboxes(imap_session: &mut IMAPSession) -> Option<ZeroCopy<Vec<Name>>> {
    let mailboxes = imap_session.list(Some(""), Some("*"));
    if !mailboxes.is_err() {
        return Some(mailboxes.unwrap())
    }

    return None
}
