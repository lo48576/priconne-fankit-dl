//! Fankit-related stuff.

use std::collections::{HashSet, VecDeque};

pub use self::{
    id::{FankitId, FankitIdParseError},
    info::FankitInfo,
    list_page_index::{FankitListPageIndex, FankitListPageIndexParseError},
};

mod id;
mod info;
mod list_page_index;

/// Common URL prefix for fankit-related pages.
const URL_FANKIT_TOP: &str = "https://priconne-redive.jp/fankit02/";

/// URL prefix for fankit items.
const URL_FANKIT_ITEM_BASE: &str = URL_FANKIT_TOP;

/// URL prefix for fankit list pages.
const URL_FANKIT_LIST_BASE: &str = "https://priconne-redive.jp/fankit02/page/";

/// Returns fankits.
pub fn get_fankits() -> Result<HashSet<FankitId>, Box<dyn std::error::Error + Send + Sync + 'static>>
{
    // Wanted to pop from `HashSet` but it is not in std hashset.
    // Using `VecDeque` instead.
    let mut list_undone: VecDeque<_> = std::iter::once(FankitListPageIndex::new(1)).collect();

    let mut fankits = HashSet::new();
    let mut list_done = HashSet::new();

    // Load the index pages.
    while let Some(list_page) = list_undone.pop_front() {
        if !list_done.insert(list_page) {
            // `list_page` is already checked.
            continue;
        }

        let (new_fankits, other_lists) = list_page.load()?;
        list_undone.extend(other_lists.into_iter().filter(|v| !list_done.contains(v)));

        fankits.extend(new_fankits);

        log::debug!(
            "List pages done = {:?}, undone = {:?}",
            list_done,
            list_undone
        );
    }

    Ok(fankits)
}
