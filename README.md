# REmailFS

## About 

REmailFS is a FUSE application allows emails to be fetched from an IMAP server and viewed as files. As on now the funcitonality that has been implemented is fairly rudimentary but includes:

1) Connecting to the IMAP server.
2) Listing all mailboxes on the server.
3) Downloading an email and viewing it as a file.

As it stands REmailFS only supports reading data from an IMAP server, modifying the contents of the server will be supported in the future. Only Gmail has been tested so far.  

REmailFS lists each mailbox as a directory in the filesystem tree. The contents of a directory will be the child mailboxes of the current mailbox and the emails that are stored in the mailbox. When an email is opened locally it is fetched from the server if it is not stored locally already and displays the subject, date, sender, and content of the email. Currently, due to variations in the format of reveived emails, the content of an email is fetched if and only if the email containts a MIME text/plain section. Any other content will be ignored.  

## FUSE 

The following FUSE methods have been implemented so far, any of the methods not listed here are default implementations.

- init
- destroy
- getattr
- lookup
- readdir
- read

## Initial Setup
IMAP must be enabled on any account that you wish to use REmailFS with.

### Gmail Setup
- Enable IMAP for [Gmail](https://mail.google.com/mail/u/0/#settings/fwdandpop)
	1) Open your [Gmail inbox](https://mail.google.com/mail/u/0/)
	2) Go to the Settings page by clicking the gear icon in the upper right hand corner and selecting Settings from the drop down menu.
	3) In the Forwarding and POP/IMAP tab click the enable IMAP button.
	4) Click save changes.
- Enable REmailFS access to your Google account with an app password
Full instructions at to generate an [App password](https://support.google.com/accounts/answer/185833)

## Running REmailFS
After following the setup steps above REmailFS can be built and run using 
`cargo run <mountpoint>` 

For more information on how to run REmailFS run the command 
`cargo run -- -h`
