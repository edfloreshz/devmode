use std::fs;
use std::path::Path;

use dirs::home_dir;
use regex::bytes::Regex;

use crate::cmd::logic::{Clone, Host};

pub fn clone<'a>(clone: &Clone) -> Result<(), Box<dyn std::error::Error>> {
    match make_repo_path(clone) {
        Ok(path) => {
            println!("Cloning {}...", clone.repo.unwrap());
            match git2::Repository::clone(clone.url().as_str(), &path) {
                Ok(_repo) => Ok(println!("{} cloned successfully to {}", clone.repo.unwrap(), path)),
                Err(e) => Err(Box::new(e)),
            }
        }
        Err(e) => Err(e),
    }
}

fn make_repo_path<'a>(clone: &Clone) -> Result<String, Box<dyn std::error::Error>> {
    let path = format!(
        "{}/Developer/{}/{}/{}",
        home_dir().unwrap().display(),
        match clone.host.as_ref().unwrap() {
            Host::GitHub(host) => host,
            Host::GitLab(host) => host,
        },
        clone.owner.as_ref().unwrap(),
        clone.repo.as_ref().unwrap()
    );
    if !Path::new(path.as_str()).exists() {
        match fs::create_dir_all(path.clone()) {
            Ok(_) => Ok(path),
            Err(e) => Err(Box::new(e))
        }
    } else {
        Ok(path)
    }
}

pub fn parse_url(url: &str, rx: Regex) -> Clone {
    let captures = rx.captures(url.as_ref()).unwrap();
    let host = captures.get(4).map(|m| std::str::from_utf8(m.as_bytes()).unwrap()).unwrap();
    let owner = captures.get(6).map(|m| std::str::from_utf8(m.as_bytes()).unwrap()).unwrap();
    let repo = captures.get(7).map(|m| std::str::from_utf8(m.as_bytes()).unwrap()).unwrap();
    Clone::new(Host::from(host), Option::from(owner), Option::from(repo))
}