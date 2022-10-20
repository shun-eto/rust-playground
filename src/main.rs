use rust_playground::entities::add::add;
use rust_playground::entities::user::User;

struct Person {
    name: String,
    age: u32,
}

impl Person {
    fn new(name: &str, age: u32) -> Person {
        Person {
            name: String::from(name),
            age,
        }
    }

    fn get_name(&self) {
        println!("{}", self.name);
    }

    fn get_age(&self) {
        println!("{}", self.age);
    }

    fn change_name(&mut self, name: &str) {
        self.name = String::from(name);
    }
}

fn main() {
    println!("Hello, world!");

    let user = User::new("test");
    let mut person = Person::new("shun", 29);
    person.get_age();
    person.get_name();
    person.change_name("eto");
    person.get_name();

    test1();

    let data = add(1, 2);
    println!("{}", data);
}

fn test1() -> String {
    todo!("test")
}

#[test]
fn test_success_add() {
    assert_eq!(1, add(1, 0))
}

#[test]
#[should_panic]
fn test_failed_add() {
    add(0, 0);
}
