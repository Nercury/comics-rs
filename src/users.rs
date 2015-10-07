use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serde_json;

use users_models::UserRepr;

pub struct Users {
    users: HashMap<String, UserRepr>,
}

impl Users {
    pub fn from_file(path: &str) -> Option<Users> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            _ => {
                println!("failed to open index storage");
                return None;
            },
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            panic!("error reading index storage {:?}", e);
        }

        let deserialized: Vec<UserRepr> = match serde_json::from_str(&contents) {
            Ok(contents) => contents,
            Err(e) => {
                panic!("error deserializing index storage {:?}", e);
            }
        };

        Some(Users {
            users: deserialized.into_iter().map(|v| (v.username.clone(), v)).collect()
        })
    }

    pub fn authorize(&self, username: &str, maybe_password: &str) -> bool {
        match self.users.get(username) {
            Some(&UserRepr { ref password, .. }) if password == maybe_password => true,
            _ => false,
        }
    }
}
