use std::{fmt::Display, mem::size_of, usize};

pub const ID_SIZE: usize = size_of::<i32>();
pub const USERNAME_SIZE: usize = 32;
pub const EMAIL_SIZE: usize = 355;

pub const ID_OFFSET: usize = 0;
pub const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
pub const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

#[derive(Debug, Clone)]
pub struct Row {
    pub id: i32,
    pub username: String,
    pub email: String,
}

impl Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "| {:<5} | {:<32} | {:<355} |",
            self.id, self.username, self.email
        )
    }
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
        let username_copy_len = username_bytes.len().min(USERNAME_SIZE);
        username_dest[..username_copy_len].copy_from_slice(&username_bytes[..username_copy_len]);

        /*
         * copy email
         * **/
        let email_bytes = self.email.as_bytes();
        let email_dst = &mut destination[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE];

        email_dst.fill(0);
        let email_copy_len = email_bytes.len().min(EMAIL_SIZE);
        email_dst[..email_copy_len].copy_from_slice(&email_bytes[..email_copy_len]);
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

/*
*
* TESTS
* ***/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let row = Row {
            id: 42,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let mut buffer = vec![0u8; ROW_SIZE];
        row.serialize(&mut buffer);

        let mut deserialized = Row::new();
        deserialized.ingest_deserialized(&buffer);

        assert_eq!(deserialized.id, row.id);
        assert_eq!(deserialized.username, row.username);
        assert_eq!(deserialized.email, row.email);
    }

    #[test]
    fn test_serialize_empty_strings() {
        let row = Row {
            id: 1,
            username: "".to_string(),
            email: "".to_string(),
        };

        let mut buffer = vec![0u8; ROW_SIZE];
        row.serialize(&mut buffer);

        let mut deserialized = Row::new();
        deserialized.ingest_deserialized(&buffer);

        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.username, "");
        assert_eq!(deserialized.email, "");
    }

    #[test]
    fn test_serialize_max_length_strings() {
        let row = Row {
            id: 1,
            username: "a".repeat(USERNAME_SIZE),
            email: "b".repeat(EMAIL_SIZE),
        };

        let mut buffer = vec![0u8; ROW_SIZE];
        row.serialize(&mut buffer);

        let mut deserialized = Row::new();
        deserialized.ingest_deserialized(&buffer);

        assert_eq!(deserialized.username.len(), USERNAME_SIZE);
        assert_eq!(deserialized.email.len(), EMAIL_SIZE);
    }

    #[test]
    fn test_serialize_truncates_long_strings() {
        let long_username = "a".repeat(100);
        let long_email = "b".repeat(500);

        let row = Row {
            id: 1,
            username: long_username.clone(),
            email: long_email.clone(),
        };

        let mut buffer = vec![0u8; ROW_SIZE];
        row.serialize(&mut buffer);

        let mut deserialized = Row::new();
        deserialized.ingest_deserialized(&buffer);

        assert_eq!(deserialized.username.len(), USERNAME_SIZE);
        assert_eq!(deserialized.email.len(), EMAIL_SIZE);
    }
}
