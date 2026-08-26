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

/// Which transport a remote URL uses — the two forms a git host offers side
/// by side, so a repo can be switched between them without retyping host,
/// owner, and name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Https,
    Ssh,
}

impl Scheme {
    /// Guesses the scheme a remote URL is already using, so a toggle has
    /// something to flip from.
    pub fn detect(url: &str) -> Self {
        if url.starts_with("git@") || url.starts_with("ssh://") {
            Scheme::Ssh
        } else {
            Scheme::Https
        }
    }

    pub fn other(self) -> Self {
        match self {
            Scheme::Https => Scheme::Ssh,
            Scheme::Ssh => Scheme::Https,
        }
    }
}

impl ParsedUrl {
    /// Renders this URL back out in the given transport, e.g. to switch an
    /// existing remote between HTTPS and SSH.
    pub fn format(&self, scheme: Scheme) -> String {
        match scheme {
            Scheme::Https => format!("https://{}/{}/{}.git", self.host, self.owner, self.name),
            Scheme::Ssh => format!("git@{}:{}/{}.git", self.host, self.owner, self.name),
        }
    }
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

    #[test]
    fn detects_scheme() {
        assert_eq!(Scheme::detect("https://github.com/torvalds/linux.git"), Scheme::Https);
        assert_eq!(Scheme::detect("git@github.com:torvalds/linux.git"), Scheme::Ssh);
        assert_eq!(Scheme::detect("ssh://git@github.com/torvalds/linux.git"), Scheme::Ssh);
    }

    #[test]
    fn formats_both_schemes_from_a_parsed_url() {
        let parsed = parse_url("https://github.com/torvalds/linux.git").unwrap();

        assert_eq!(parsed.format(Scheme::Https), "https://github.com/torvalds/linux.git");
        assert_eq!(parsed.format(Scheme::Ssh), "git@github.com:torvalds/linux.git");
    }
}
