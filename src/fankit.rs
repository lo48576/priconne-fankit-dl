//! Fankit-related stuff.

use std::{
    collections::{HashSet, VecDeque},
    time::Duration,
};

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

/// Returns fankits if new fankit is detected.
pub fn get_fankits_if_new_fankit_found(
    known_fankits: impl IntoIterator<Item = FankitId>,
    crawl_delay: Duration,
) -> Result<Option<HashSet<FankitId>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use std::iter::FromIterator;

    const FIRST_PAGE_INDEX: FankitListPageIndex = FankitListPageIndex::new(1);

    let (new_fankits, other_lists) = FIRST_PAGE_INDEX.load()?;
    let new_fankits = HashSet::from_iter(new_fankits);
    let known_fankits = HashSet::from_iter(known_fankits);

    if new_fankits.is_subset(&known_fankits) {
        // There are no new fankits.
        return Ok(None);
    }

    // Wanted to pop from `HashSet` but it is not in std hashset.
    // Using `VecDeque` instead.
    let mut list_undone = VecDeque::from_iter(other_lists);

    let mut fankits = new_fankits;
    let mut list_done = std::iter::once(FIRST_PAGE_INDEX).collect::<HashSet<_>>();

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

        log::debug!("Sleeping for {:?}", crawl_delay);
        std::thread::sleep(crawl_delay);
    }

    Ok(Some(fankits))
}
