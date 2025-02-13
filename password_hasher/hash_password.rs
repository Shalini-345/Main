use bcrypt::{hash, DEFAULT_COST};

fn main() {
    let password = "newpassword"; // Change this to match your existing password
    let hashed_password = hash(password, DEFAULT_COST).unwrap();
    println!("Hashed password: {}", hashed_password);
}
