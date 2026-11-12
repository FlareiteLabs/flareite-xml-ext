//! XML's code: closing tags typed for the user (see [`tags`]):
//! `</item>` after `<item>`, and the rest of `</item>` after `</`.

mod tags;

use flareite_extension_api::language_features::{AutoInsertRequest, Insertion};
use flareite_extension_api::types::{Activation, Position, Range};
use flareite_extension_api::{Extension, host, register};

/// A closing tag to insert as `>` or `/` is typed, if one is due.
fn auto_insert(request: &AutoInsertRequest) -> Option<Insertion> {
  let trigger = request.trigger.chars().next()?;
  let range = Range {
    start: Position { line: 0, column: 0 },
    end: request.position,
  };
  // The text up to the cursor, as it was when the trigger was typed.
  let before = host::read_range(request.document, range).ok()?;
  if before.version != request.version {
    return None;
  }
  let (text, cursor) = tags::auto_insert(&before.text, trigger)?;
  Some(Insertion {
    text,
    cursor: u32::try_from(cursor).ok()?,
  })
}

struct Xml;

impl Extension for Xml {
  fn activate(_: Activation) -> Result<Self, String> {
    Ok(Xml)
  }

  fn provide_auto_insert(
    &mut self,
    request: AutoInsertRequest,
  ) -> Result<Option<Insertion>, String> {
    Ok(auto_insert(&request))
  }
}

register!(Xml);
