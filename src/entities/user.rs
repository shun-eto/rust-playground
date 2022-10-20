pub struct User {
    name: String,
}

impl User {
    pub fn new(name: &str) -> User {
        User {
            name: String::from(name),
        }
    }
}
