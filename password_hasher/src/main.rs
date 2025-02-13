use bcrypt::{hash, DEFAULT_COST};

fn main() {
    let password = "newpassword"; // Change this to your desired password
    let hashed_password = hash(password, DEFAULT_COST).unwrap();
    println!("Hashed password: {}", hashed_password);
}

