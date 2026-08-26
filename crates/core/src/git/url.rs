use crate::error::{Error, Result};

/// A git URL decomposed into the pieces devmode's path layout templates need.
/// Works for both HTTPS (`https://host/owner/repo.git`) and SSH
/// (`git@host:owner/repo.git`) remote URLs.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
    pub host: String,
    pub owner: String,
    pub name: String,
    /// A non-default SSH port, e.g. the `2222` in
    /// `ssh://git@host:2222/owner/repo.git` for a self-hosted git server.
    /// Only ever set from an SSH URL: a host's HTTPS port isn't derivable
    /// from its SSH one, so this is never applied to an HTTPS URL.
    pub ssh_port: Option<u16>,
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
    ///
    /// A custom SSH port only ever applies to the SSH form. Scp-style SSH
    /// (`git@host:owner/repo.git`) has no syntax for a port at all, so a
    /// ported SSH remote uses the long `ssh://` form instead — the only way
    /// git itself accepts a non-default SSH port.
    pub fn format(&self, scheme: Scheme) -> String {
        match scheme {
            Scheme::Https => format!("https://{}/{}/{}.git", self.host, self.owner, self.name),
            Scheme::Ssh => match self.ssh_port {
                Some(port) => format!("ssh://git@{}:{port}/{}/{}.git", self.host, self.owner, self.name),
                None => format!("git@{}:{}/{}.git", self.host, self.owner, self.name),
            },
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
    let ssh_port = matches!(
        parsed.scheme,
        git_url_parse::Scheme::Ssh | git_url_parse::Scheme::GitSsh
    )
    .then_some(parsed.port)
    .flatten();

    Ok(ParsedUrl {
        host,
        owner,
        name: parsed.name,
        ssh_port,
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
                ssh_port: None,
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
                ssh_port: None,
            }
        );
    }

    #[test]
    fn parses_a_port_from_the_long_ssh_form() {
        let parsed =
            parse_url("ssh://git@code.edfloreshz.dev:2222/edfloreshz/plutotv-downloader.git").unwrap();
        assert_eq!(
            parsed,
            ParsedUrl {
                host: "code.edfloreshz.dev".to_string(),
                owner: "edfloreshz".to_string(),
                name: "plutotv-downloader".to_string(),
                ssh_port: Some(2222),
            }
        );
    }

    #[test]
    fn ignores_a_port_on_an_https_url() {
        // An HTTPS port isn't an SSH port, and reformatting to SSH shouldn't
        // invent one from it.
        let parsed =
            parse_url("https://code.edfloreshz.dev:8443/edfloreshz/plutotv-downloader.git").unwrap();
        assert_eq!(parsed.ssh_port, None);
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

    #[test]
    fn a_ported_ssh_remote_keeps_its_port_in_ssh_but_not_https() {
        let parsed =
            parse_url("ssh://git@code.edfloreshz.dev:2222/edfloreshz/plutotv-downloader.git").unwrap();

        // Scp-style SSH has no slot for a port, so a ported remote has to
        // stay in the long `ssh://` form rather than silently drop it.
        assert_eq!(
            parsed.format(Scheme::Ssh),
            "ssh://git@code.edfloreshz.dev:2222/edfloreshz/plutotv-downloader.git"
        );
        // The SSH port says nothing about where HTTPS is served, so it's
        // dropped rather than guessed at.
        assert_eq!(
            parsed.format(Scheme::Https),
            "https://code.edfloreshz.dev/edfloreshz/plutotv-downloader.git"
        );
    }
}
