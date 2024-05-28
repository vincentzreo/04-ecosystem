use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, Serialize, Deserialize, PartialEq)]
struct User {
    #[builder(setter(into))]
    name: String,
    age: u8,
    dob: DateTime<Utc>,
    #[builder(setter(each(name = "skill", into)))]
    skills: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let user = UserBuilder::default()
        .name("Alice")
        .age(20)
        .dob(Utc::now())
        .skill("Rust")
        .skill("Python")
        .build()?;
    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    let user1: User = serde_json::from_str(&json)?;
    println!("{:?}", user1);

    assert_eq!(user, user1);
    Ok(())
}
