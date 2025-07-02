use bcrypt::{hash, DEFAULT_COST};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Change "fake" to whatever password you want to hash
    let plain = "fake";
    let hashed = hash(plain, DEFAULT_COST)?;
    println!("{}", hashed);
    Ok(())
}