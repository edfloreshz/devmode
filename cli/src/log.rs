use colored::*;

pub fn success(msg: &str) {
    println!("{} {}", "✅ ".green(), msg.green());
}

pub fn warning(msg: &str) {
    println!("{} {}", "⚠️ ".yellow(), msg.yellow());
}

pub fn error(msg: &str) {
    println!("{} {}", "⛔ ".red(), msg.red());
}

pub fn info(msg: &str) {
    println!("{} {}", "ℹ️ ".cyan(), msg.cyan());
}
