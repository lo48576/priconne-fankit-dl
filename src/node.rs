//! Generic node utils.

use std::collections::HashSet;

use html5ever::{parse_document, tree_builder::Attribute};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

pub use self::traverse::Traverse;

mod traverse;

/// Returns hrefs of anchors in the given URL.
pub fn get_anchors(node: Handle) -> impl Iterator<Item = String> {
    // Get anchors.
    // Collect to `HashSet<_>` to deduplicate.
    // allow(clippy::mutable_key_type): This is a hashset of tendrils, so allowing
    // interior mutability here is safe.
    #[allow(clippy::mutable_key_type)]
    let anchors = Traverse::new(node)
        .filter_map(|node| match &node.data {
            NodeData::Element { name, attrs, .. } => {
                if &name.local == "a" {
                    // Found an anchor.
                    attrs
                        .borrow()
                        .iter()
                        .find(|attr| &attr.name.local == "href")
                        .map(|attr| attr.value.clone())
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<HashSet<_>>();

    anchors.into_iter().map(|href| href.to_string())
}

pub fn load_dom(url: &str) -> Result<RcDom, Box<dyn std::error::Error + Send + Sync + 'static>> {
    log::trace!("Loading page: {:?}", url);
    let dom = {
        use html5ever::tendril::stream::TendrilSink;

        let top_text = reqwest::blocking::get(url)?.error_for_status()?.text()?;
        parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut std::io::Cursor::new(&top_text))?
    };
    log::trace!("DOM errors for {:?}: {:#?}", url, dom.errors);

    Ok(dom)
}

pub fn attrs_has_id<'a>(id: &str, attrs: impl IntoIterator<Item = &'a Attribute>) -> bool {
    attrs
        .into_iter()
        .any(|attr| &attr.name.local == "id" && &*attr.value == id)
}

pub fn node_has_id(id: &str, node: &Handle) -> bool {
    match &node.data {
        NodeData::Element { attrs, .. } => attrs_has_id(id, attrs.borrow().iter()),
        _ => false,
    }
}

pub fn attrs_has_class<'a>(class: &str, attrs: impl IntoIterator<Item = &'a Attribute>) -> bool {
    attrs.into_iter().any(|attr| {
        &attr.name.local == "class" && attr.value.split_ascii_whitespace().any(|c| c == class)
    })
}

pub fn node_has_class(class: &str, node: &Handle) -> bool {
    match &node.data {
        NodeData::Element { attrs, .. } => attrs_has_class(class, attrs.borrow().iter()),
        _ => false,
    }
}

pub fn inner_text(node: Handle) -> String {
    let mut buf = String::new();
    for node in Traverse::new(node) {
        if let NodeData::Text { contents } = &node.data {
            buf.push_str(&**contents.borrow());
        }
    }
    buf
}
