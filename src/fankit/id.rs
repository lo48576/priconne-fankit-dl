//! Fankit ID.

use std::{error, fmt};

use crate::{
    fankit::{FankitInfo, URL_FANKIT_ITEM_BASE},
    node::load_dom,
};

/// Fankit ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FankitId(usize);

impl FankitId {
    /// Creates a new `FankitId`.
    pub(crate) fn new(v: usize) -> Self {
        Self(v)
    }

    /// Returns the URL of the fankit.
    pub fn to_url(self) -> String {
        format!("{}{}/", URL_FANKIT_ITEM_BASE, self.0)
    }

    /// Returns `usize` value.
    pub fn to_usize(self) -> usize {
        self.0
    }

    /// Loads the fankit page, and returns a metadata and the image URLs.
    pub fn load(self) -> Result<FankitInfo, Box<dyn error::Error + Send + Sync + 'static>> {
        log::trace!("Loading fankit page: {:?}", self);
        let dom = load_dom(&self.to_url())?;
        FankitInfo::from_node(self, dom.document.clone())
    }
}

impl std::str::FromStr for FankitId {
    type Err = FankitIdParseError;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        if !url.starts_with(URL_FANKIT_ITEM_BASE) {
            return Err(FankitIdParseError::BaseMismatch);
        }
        let relpath = url[URL_FANKIT_ITEM_BASE.len()..].trim_end_matches('/');
        relpath
            .parse::<usize>()
            .map(Self::new)
            .map_err(|_| FankitIdParseError::InvalidPath)
    }
}

/// `FankitListPageIndex` parse error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FankitIdParseError {
    /// Base URL mismatch.
    BaseMismatch,
    /// Invalid path.
    InvalidPath,
}

impl error::Error for FankitIdParseError {}

impl fmt::Display for FankitIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaseMismatch => f.write_str("Base URL mismatch"),
            Self::InvalidPath => f.write_str("Invalid path"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_url() {
        assert_eq!(
            FankitId::new(1234).to_url(),
            "https://priconne-redive.jp/fankit02/1234/"
        );
    }

    #[test]
    fn list_index() {
        assert_eq!(
            "https://priconne-redive.jp/fankit02/1234/"
                .parse::<FankitId>()
                .ok(),
            Some(FankitId::new(1234))
        );
        assert!("https://priconne-redive.jp/fankit02/page/4/"
            .parse::<FankitId>()
            .is_err());
        assert!("https://priconne-redive.jp/".parse::<FankitId>().is_err());
    }
}
