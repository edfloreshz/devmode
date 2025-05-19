use crate::log;

// Helper to confirm the user wants to overwrite an existing repository.
pub fn overwrite() -> bool {
    log::warning("Found existing repository, overwrite it? y/n");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim(), "y" | "Y")
}
