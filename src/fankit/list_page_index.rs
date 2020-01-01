//! Fankit list page index.

use std::{error, fmt};

use crate::{
    fankit::{FankitId, URL_FANKIT_LIST_BASE, URL_FANKIT_TOP},
    node::{get_anchors, load_dom},
};

/// Fankit list page index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FankitListPageIndex(usize);

impl FankitListPageIndex {
    /// Creates a new `FankitListPageIndex`.
    pub const fn new(v: usize) -> Self {
        Self(v)
    }

    /// Returns the URL of the fankit list page.
    pub fn to_url(self) -> String {
        if self.0 <= 1 {
            URL_FANKIT_TOP.to_owned()
        } else {
            format!("{}{}/", URL_FANKIT_LIST_BASE, self.0)
        }
    }

    /// Loads the list page, and returns the fankit ids and other list pages
    /// found.
    pub fn load(
        self,
    ) -> Result<(Vec<FankitId>, Vec<Self>), Box<dyn error::Error + Send + Sync + 'static>> {
        log::trace!("Loading list page: {:?}", self);
        let dom = load_dom(&self.to_url())?;

        let mut fankits = Vec::new();
        let mut list_pages = Vec::new();
        for href in get_anchors(dom.document).filter(|href| href.starts_with(URL_FANKIT_TOP)) {
            if let Ok(fankit) = href.parse::<FankitId>() {
                fankits.push(fankit);
            } else if let Ok(list_page) = href.parse::<FankitListPageIndex>() {
                list_pages.push(list_page);
            }
        }

        Ok((fankits, list_pages))
    }
}

impl std::str::FromStr for FankitListPageIndex {
    type Err = FankitListPageIndexParseError;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        if url == URL_FANKIT_TOP {
            return Ok(Self::new(1));
        }
        if !url.starts_with(URL_FANKIT_LIST_BASE) {
            return Err(FankitListPageIndexParseError::BaseMismatch);
        }
        let relpath = url[URL_FANKIT_LIST_BASE.len()..].trim_end_matches('/');
        relpath
            .parse::<usize>()
            .map(Self::new)
            .map_err(|_| FankitListPageIndexParseError::InvalidPath)
    }
}

/// `FankitListPageIndex` parse error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FankitListPageIndexParseError {
    /// Base URL mismatch.
    BaseMismatch,
    /// Invalid path.
    InvalidPath,
}

impl error::Error for FankitListPageIndexParseError {}

impl fmt::Display for FankitListPageIndexParseError {
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
            FankitListPageIndex::new(1).to_url(),
            "https://priconne-redive.jp/fankit02/"
        );
        assert_eq!(
            FankitListPageIndex::new(3).to_url(),
            "https://priconne-redive.jp/fankit02/page/3/"
        );
    }

    #[test]
    fn from_list_index() {
        assert_eq!(
            "https://priconne-redive.jp/fankit02/"
                .parse::<FankitListPageIndex>()
                .ok(),
            Some(FankitListPageIndex::new(1))
        );
        assert_eq!(
            "https://priconne-redive.jp/fankit02/page/3/"
                .parse::<FankitListPageIndex>()
                .ok(),
            Some(FankitListPageIndex::new(3))
        );
        assert_eq!(
            "https://priconne-redive.jp/fankit02/page/4/"
                .parse::<FankitListPageIndex>()
                .ok(),
            Some(FankitListPageIndex::new(4))
        );
        assert!("https://priconne-redive.jp/"
            .parse::<FankitListPageIndex>()
            .is_err());
    }
}
