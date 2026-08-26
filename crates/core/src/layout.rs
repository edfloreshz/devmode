use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Where a repo lands under the configured clone root. Built-in variants
/// cover common conventions; `Custom` lets a user supply their own
/// `{host}`/`{owner}`/`{repo}` template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PathLayout {
    /// `{host}/{owner}/{repo}` — e.g. `github.com/torvalds/linux`.
    HostOwnerRepo,
    /// `{owner}/{repo}` — e.g. `torvalds/linux`.
    OwnerRepo,
    /// `{repo}` — everything directly under the clone root.
    Flat,
    /// A user-supplied template using the same `{host}`/`{owner}`/`{repo}` placeholders.
    Custom { template: String },
}

impl Default for PathLayout {
    fn default() -> Self {
        Self::HostOwnerRepo
    }
}

impl PathLayout {
    fn template(&self) -> &str {
        match self {
            Self::HostOwnerRepo => "{host}/{owner}/{repo}",
            Self::OwnerRepo => "{owner}/{repo}",
            Self::Flat => "{repo}",
            Self::Custom { template } => template,
        }
    }

    /// Renders this layout into a relative path for the given host/owner/repo.
    pub fn render(&self, host: &str, owner: &str, repo: &str) -> PathBuf {
        let rendered = self
            .template()
            .replace("{host}", host)
            .replace("{owner}", owner)
            .replace("{repo}", repo);
        PathBuf::from(rendered)
    }

    /// Parses the `dm config set layout <value>` argument: one of the
    /// built-in names, or `custom:<template>` for a user-defined template.
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "host_owner_repo" => Ok(Self::HostOwnerRepo),
            "owner_repo" => Ok(Self::OwnerRepo),
            "flat" => Ok(Self::Flat),
            other => match other.strip_prefix("custom:") {
                Some(template) if !template.is_empty() => Ok(Self::Custom {
                    template: template.to_string(),
                }),
                _ => Err(Error::InvalidPathLayout(other.to_string())),
            },
        }
    }

    /// Renders back to the same string form `parse` accepts, for `dm config get layout`.
    pub fn to_config_string(&self) -> String {
        match self {
            Self::HostOwnerRepo => "host_owner_repo".to_string(),
            Self::OwnerRepo => "owner_repo".to_string(),
            Self::Flat => "flat".to_string(),
            Self::Custom { template } => format!("custom:{template}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_built_in_layouts() {
        assert_eq!(
            PathLayout::HostOwnerRepo.render("github.com", "torvalds", "linux"),
            PathBuf::from("github.com/torvalds/linux")
        );
        assert_eq!(
            PathLayout::OwnerRepo.render("github.com", "torvalds", "linux"),
            PathBuf::from("torvalds/linux")
        );
        assert_eq!(
            PathLayout::Flat.render("github.com", "torvalds", "linux"),
            PathBuf::from("linux")
        );
    }

    #[test]
    fn renders_custom_template() {
        let layout = PathLayout::Custom {
            template: "{owner}-{repo}".to_string(),
        };
        assert_eq!(
            layout.render("github.com", "torvalds", "linux"),
            PathBuf::from("torvalds-linux")
        );
    }

    #[test]
    fn parses_and_round_trips() {
        for value in ["host_owner_repo", "owner_repo", "flat", "custom:{repo}"] {
            let layout = PathLayout::parse(value).unwrap();
            assert_eq!(layout.to_config_string(), value);
        }
    }

    #[test]
    fn rejects_empty_custom_template() {
        assert!(PathLayout::parse("custom:").is_err());
        assert!(PathLayout::parse("nonsense").is_err());
    }
}
