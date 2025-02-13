use bcrypt::{hash, DEFAULT_COST};

fn main() {
    let password = "newpassword"; // The password you need to hash
    let hashed_password = hash(password, DEFAULT_COST).unwrap();
    println!("Hashed password: {}", hashed_password);
}
