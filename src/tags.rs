//! Closing tags typed for the user, as VS Code's HTML support does them:
//! typing the `>` of a start tag such as `<item id="1">` adds `</item>`
//! after the cursor, and typing `</` finishes the innermost open element's
//! end tag (`</item>`).
//!
//! The text before the cursor is scanned as XML is, closely enough for
//! this: comments, CDATA sections, processing instructions (`<?xml ...?>`),
//! declarations (`<!DOCTYPE ...>`), quoted attribute values, and
//! empty-element tags (`<x />`). Names are matched with their case.

fn is_name_char(c: u8) -> bool {
  c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b':' | b'.') || c >= 0x80
}

fn is_name_start(c: u8) -> bool {
  c.is_ascii_alphabetic() || c == b'_' || c == b':' || c >= 0x80
}

/// What to insert after `trigger` was typed at the end of `before` (the
/// document's text up to the cursor): the text, and where in it to leave
/// the cursor, in bytes. `None` for nothing.
pub fn auto_insert(before: &str, trigger: char) -> Option<(String, usize)> {
  let scan = scan(before);
  match trigger {
    '>' => {
      let name = scan.start_tag_ended_at_end?;
      Some((format!("</{name}>"), 0))
    }
    '/' if before.ends_with("</") && scan.end_tag_opened_at_end => {
      let name = scan.open.last()?;
      let text = format!("{name}>");
      let cursor = text.len();
      Some((text, cursor))
    }
    _ => None,
  }
}

/// What scanning the text before the cursor found.
#[derive(Debug, Default)]
struct Scan {
  /// The elements open at the end, outermost first.
  open: Vec<String>,
  /// The name of a start tag whose `>` ends the text.
  start_tag_ended_at_end: Option<String>,
  /// Whether the text ends with `</` that begins an end tag.
  end_tag_opened_at_end: bool,
}

impl Scan {
  /// Nothing to insert, with these elements open.
  fn nothing(open: Vec<String>) -> Self {
    Self {
      open,
      ..Self::default()
    }
  }
}

/// Where `close` ends after `from`, or `None` if it doesn't come.
fn past(text: &str, from: usize, close: &str) -> Option<usize> {
  text[from..]
    .find(close)
    .map(|found| from + found + close.len())
}

fn scan(text: &str) -> Scan {
  let bytes = text.as_bytes();
  let mut scan = Scan::default();
  let mut at = 0;
  while let Some(found) = text[at..].find('<') {
    let start = at + found;
    let rest = &text[start..];
    let skipped = if rest.starts_with("<!--") {
      Some(past(text, start + 4, "-->"))
    } else if rest.starts_with("<![CDATA[") {
      Some(past(text, start + 9, "]]>"))
    } else if rest.starts_with("<?") {
      Some(past(text, start + 2, "?>"))
    } else if rest.starts_with("<!") {
      Some(past(text, start + 2, ">"))
    } else {
      None
    };
    if let Some(end) = skipped {
      match end {
        Some(end) => at = end,
        // In a comment, a section, or a declaration to the end.
        None => return Scan::nothing(scan.open),
      }
      continue;
    }
    if rest.starts_with("</") {
      let name_start = start + 2;
      let mut name_end = name_start;
      while name_end < bytes.len() && is_name_char(bytes[name_end]) {
        name_end += 1;
      }
      if name_end == bytes.len() {
        scan.end_tag_opened_at_end = name_end == name_start;
        return scan;
      }
      let name = &text[name_start..name_end];
      if let Some(index) = scan.open.iter().rposition(|open| open == name) {
        scan.open.truncate(index);
      }
      match past(text, name_end, ">") {
        Some(end) => at = end,
        None => return Scan::nothing(scan.open),
      }
      continue;
    }
    if !bytes.get(start + 1).is_some_and(|&c| is_name_start(c)) {
      at = start + 1;
      continue;
    }
    let name_start = start + 1;
    let mut name_end = name_start;
    while name_end < bytes.len() && is_name_char(bytes[name_end]) {
      name_end += 1;
    }
    let name = &text[name_start..name_end];
    // The attributes, up to the tag's `>`, past quoted values.
    let mut quote = None;
    let mut closed = None;
    for (i, &c) in bytes.iter().enumerate().skip(name_end) {
      match quote {
        Some(q) if c == q => quote = None,
        Some(_) => {}
        None if c == b'"' || c == b'\'' => quote = Some(c),
        None if c == b'>' => {
          closed = Some(i);
          break;
        }
        None => {}
      }
    }
    let Some(end) = closed else {
      // Still in the tag.
      return Scan::nothing(scan.open);
    };
    at = end + 1;
    if end > name_end && bytes[end - 1] == b'/' {
      // An empty element.
      continue;
    }
    scan.open.push(name.to_owned());
    if at == bytes.len() {
      scan.start_tag_ended_at_end = Some(name.to_owned());
      return scan;
    }
  }
  scan
}

#[cfg(test)]
mod tests {
  use super::*;

  fn after_gt(before: &str) -> Option<String> {
    auto_insert(before, '>').map(|(text, cursor)| {
      assert_eq!(cursor, 0);
      text
    })
  }

  fn after_slash(before: &str) -> Option<String> {
    auto_insert(before, '/').map(|(text, cursor)| {
      assert_eq!(cursor, text.len());
      text
    })
  }

  #[test]
  fn closes_start_tags() {
    assert_eq!(after_gt("<root>").as_deref(), Some("</root>"));
    assert_eq!(
      after_gt("<?xml version=\"1.0\"?>\n<root>\n  <item id='a>b'>").as_deref(),
      Some("</item>")
    );
    assert_eq!(
      after_gt("<xsl:template>").as_deref(),
      Some("</xsl:template>")
    );
    assert_eq!(after_gt("<Item>").as_deref(), Some("</Item>"));
    assert_eq!(
      after_gt("<br>").as_deref(),
      Some("</br>"),
      "nothing is void"
    );
  }

  #[test]
  fn leaves_empty_elements_and_what_isnt_a_tag() {
    assert_eq!(after_gt("<item/>"), None);
    assert_eq!(after_gt("<item a=\"1\" />"), None);
    assert_eq!(after_gt("<root></root>"), None);
    assert_eq!(after_gt("<?xml version=\"1.0\"?>"), None);
    assert_eq!(after_gt("<!DOCTYPE note>"), None);
    assert_eq!(after_gt("<!-- <item>"), None);
    assert_eq!(after_gt("<![CDATA[ <item>"), None);
    assert_eq!(after_gt("<a title=\"<b>"), None);
    assert_eq!(after_gt("a > b"), None);
  }

  #[test]
  fn finishes_end_tags() {
    assert_eq!(after_slash("<root>\n  </").as_deref(), Some("root>"));
    assert_eq!(after_slash("<a><b>x</b></").as_deref(), Some("a>"));
    assert_eq!(after_slash("<a><B></b></").as_deref(), Some("B>"));
    assert_eq!(
      after_slash("<a><![CDATA[ </b> ]]></").as_deref(),
      Some("a>")
    );
    assert_eq!(after_slash("</"), None);
    assert_eq!(after_slash("<a></a></"), None);
    assert_eq!(after_slash("<a><!-- </"), None);
  }
}
