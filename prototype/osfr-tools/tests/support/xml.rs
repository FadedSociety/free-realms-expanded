//! One client XML id reader for the content gates.
//!
//! The boombox, Halloween and NPC ambient gates all check content ids against a
//! client XML table; one parser keeps a fix from landing in only one gate (`<AnimationSlot`
//! matching `<AnimationSlotGroup>`, a commented-out element, a self-closing tag).
//! Test-only: production never reads client resources.

use std::{collections::HashSet, fs, path::Path};

/// Every `id="N"` of a `<tag ...>` element in `path`, skipping XML comments.
///
/// The character after the tag name must end it, so `<AnimationSlot` cannot match
/// `<AnimationSlotGroup>`.
pub fn ids(path: &Path, tag: &str) -> HashSet<i32> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let text = without_comments(&text);
    let open = format!("<{tag}");
    let mut ids = HashSet::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find(&open) {
        let tail = &rest[at + open.len()..];
        if tail.starts_with(|c: char| c.is_whitespace() || c == '>' || c == '/') {
            let element = tail.split('>').next().unwrap_or(tail);
            if let Some(id) = attribute(element, "id").and_then(|id| id.parse().ok()) {
                ids.insert(id);
            }
        }
        rest = tail;
    }
    ids
}

fn attribute<'a>(element: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = element.find(&needle)? + needle.len();
    let end = element[start..].find('"')? + start;
    Some(&element[start..end])
}

fn without_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start + 4..].find("-->") {
            Some(end) => rest = &rest[start + 4 + end + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::ids;
    use std::{collections::HashSet, fs};

    #[test]
    fn a_tag_prefix_and_a_comment_are_not_elements() {
        let text = "<AnimationSlotGroup id=\"7\"/>\n\
                    <!-- <AnimationSlot id=\"8\"/> -->\n\
                    <AnimationSlot id=\"3302\" name=\"emo_applaud\" />";
        let path = std::env::temp_dir().join(format!(
            "osfr-xml-support-{}-{}.xml",
            std::process::id(),
            std::line!()
        ));
        fs::write(&path, text).unwrap();
        let found = ids(&path, "AnimationSlot");
        let _ = fs::remove_file(&path);
        assert_eq!(found, HashSet::from([3302]));
    }
}
