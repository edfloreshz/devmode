use std::borrow::{Borrow, Cow};
use std::fs;

use dirs::home_dir;

use crate::cmd::logic::{Clone, Host};

pub fn clone<'a>(clone: &Clone) -> Result<(), Box<dyn std::error::Error>> {
    match make_repo_path(clone) {
        Ok(path) => {
            println!("{}", path);
            match git2::Repository::clone(clone.url().as_str(), path) {
                Ok(_repo) => Ok(()),
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
    fs::create_dir_all(path.clone()).unwrap_or_else(|e| panic!("Error creating dir: {}", e));
    Ok(path)
}

fn parse_url(_url: &str) -> Clone {
    Clone::new(None, None, None)
}
