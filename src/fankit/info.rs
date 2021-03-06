//! Fankit info.

use std::collections::HashSet;

use markup5ever_rcdom::Handle;

use crate::{
    fankit::FankitId,
    node::{get_anchors, inner_text, node_has_class, node_has_id, Traverse},
};

/// Fankit info.
#[derive(Debug, Clone)]
pub struct FankitInfo {
    /// ID.
    id: FankitId,
    /// Fankit type.
    ty: String,
    /// Title.
    title: String,
    /// Image URLs.
    image_urls: HashSet<String>,
}

impl FankitInfo {
    /// Returns the item name.
    pub fn item_name(&self) -> String {
        format!("{}-{}-{}", self.id.to_usize(), self.ty, self.title)
    }

    /// Returns an iterator of image URLs.
    pub fn image_urls(&self) -> impl Iterator<Item = &str> {
        self.image_urls.iter().map(String::as_str)
    }

    pub(crate) fn from_node(
        id: FankitId,
        node: Handle,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        // Node with ID value `contents`.
        //
        // This `.clone()` for `node` should NOT be removed, because the `node` has all document
        // content with refcount.
        // Dropping `node` here causes whole trees node to be dropped except for `contents_elem`.
        #[allow(clippy::redundant_clone)]
        let contents_elem = Traverse::new(node.clone())
            .find(|node| node_has_id("contents", node))
            .ok_or("Failed to get contents element")?;

        let fankit_type_elem = Traverse::new(contents_elem.clone())
            .find(|node| node_has_class("fankit-type", node))
            .ok_or("Failed to get fankit type")?;
        let ty = inner_text(fankit_type_elem)
            .replace(char::is_whitespace, " ")
            .trim()
            .to_owned();

        let fankit_title_elem = Traverse::new(contents_elem.clone())
            .find(|node| node_has_class("title", node))
            .ok_or("Failed to get fankit title")?;
        let title = inner_text(fankit_title_elem)
            .replace(char::is_whitespace, " ")
            .trim()
            .to_owned();

        let image_urls = get_anchors(contents_elem)
            .filter(|url| url.ends_with(".jpg") || url.ends_with(".png"))
            .map(|url| url.trim().to_owned())
            .collect();

        Ok(Self {
            id,
            ty,
            title,
            image_urls,
        })
    }
}
