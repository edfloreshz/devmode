use crate::error::{Error, Result};

/// A git URL decomposed into the pieces devmode's path layout templates need.
/// Works for both HTTPS (`https://host/owner/repo.git`) and SSH
/// (`git@host:owner/repo.git`) remote URLs.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
    pub host: String,
    pub owner: String,
    pub name: String,
}

pub fn parse_url(url: &str) -> Result<ParsedUrl> {
    let parsed = git_url_parse::GitUrl::parse(url).map_err(|e| Error::InvalidGitUrl {
        url: url.to_string(),
        reason: e.to_string(),
    })?;
    let host = parsed.host.ok_or_else(|| Error::InvalidGitUrl {
        url: url.to_string(),
        reason: "could not determine host".to_string(),
    })?;
    let owner = parsed.owner.ok_or_else(|| Error::InvalidGitUrl {
        url: url.to_string(),
        reason: "could not determine owner".to_string(),
    })?;
    Ok(ParsedUrl {
        host,
        owner,
        name: parsed.name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_https_url() {
        let parsed = parse_url("https://github.com/torvalds/linux.git").unwrap();
        assert_eq!(
            parsed,
            ParsedUrl {
                host: "github.com".to_string(),
                owner: "torvalds".to_string(),
                name: "linux".to_string(),
            }
        );
    }

    #[test]
    fn parses_ssh_url() {
        let parsed = parse_url("git@github.com:torvalds/linux.git").unwrap();
        assert_eq!(
            parsed,
            ParsedUrl {
                host: "github.com".to_string(),
                owner: "torvalds".to_string(),
                name: "linux".to_string(),
            }
        );
    }
}
