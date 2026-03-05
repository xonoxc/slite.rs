use std::{mem::size_of, usize};

pub const ID_SIZE: usize = size_of::<i32>();
pub const USERNAME_SIZE: usize = 32;
pub const EMAIL_SIZE: usize = 355;

pub const ID_OFFSET: usize = 0;
pub const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
pub const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

#[derive(Debug)]
pub struct Row {
    pub id: i32,
    pub username: String,
    pub email: String,
}

impl Row {
    pub fn new() -> Self {
        Self {
            id: 0,
            username: "".to_string(),
            email: "".to_string(),
        }
    }

    pub fn serialize(&self, destination: &mut [u8]) {
        /*
         * copy id
         * **/
        destination[ID_OFFSET..ID_OFFSET + ID_SIZE].copy_from_slice(&self.id.to_le_bytes());

        let username_bytes = self.username.as_bytes();
        let username_dest = &mut destination[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE];

        username_dest.fill(0);
        username_dest[..username_bytes.len()].copy_from_slice(username_bytes);

        /*
         * copy email
         * **/
        let email_bytes = self.email.as_bytes();
        let email_dst = &mut destination[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE];

        email_dst.fill(0);
        email_dst[..email_bytes.len()].copy_from_slice(email_bytes);
    }

    pub fn ingest_deserialized(&mut self, source: &[u8]) {
        let id = i32::from_le_bytes(source[ID_OFFSET..ID_OFFSET + ID_SIZE].try_into().unwrap());

        let username_bytes = &source[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE];
        let username = String::from_utf8(
            username_bytes
                .iter()
                .copied()
                .take_while(|b| *b != 0)
                .collect(),
        )
        .unwrap();

        let email_btyes = &source[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE];
        let email = String::from_utf8(
            email_btyes
                .iter()
                .copied()
                .take_while(|b| *b != 0)
                .collect(),
        )
        .unwrap();

        self.id = id;
        self.username = username;
        self.email = email;
    }
}
