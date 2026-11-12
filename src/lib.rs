//! XML's code: closing tags typed for the user (API 1.19, see [`tags`]):
//! `</item>` after `<item>`, and the rest of `</item>` after `</`.

mod tags;

use flareite_extension_api::types::{
  Activation, CommandContext, DocumentChange, DocumentId, DocumentInfo, LogLevel, Position, Range,
};
use flareite_extension_api::{Guest, export_extension, host, language_features};

/// The command that answers what to insert as `>` or `/` is typed.
const AUTO_INSERT: &str = "flareite.xml.autoInsert";

/// Answers an auto-insert request with a closing tag, if one is due.
fn auto_insert() {
  let Some(request) = language_features::inserting() else {
    return;
  };
  let Some(trigger) = request.trigger.chars().next() else {
    return;
  };
  let range = Range {
    start: Position { line: 0, column: 0 },
    end: request.position,
  };
  // The text up to the cursor, as it was when the trigger was typed.
  let Ok(before) = host::read_range(request.document, range) else {
    return;
  };
  if before.version != request.version {
    return;
  }
  if let Some((text, cursor)) = tags::auto_insert(&before.text, trigger)
    && let Err(problem) = language_features::set_insert(&text, cursor as u32)
  {
    host::log(
      LogLevel::Error,
      &format!("can't insert a closing tag: {problem:?}"),
    );
  }
}

struct Xml;

impl Guest for Xml {
  fn activate(_: Activation) -> Result<(), String> {
    Ok(())
  }

  fn deactivate() {}

  fn run_command(command: String, _: CommandContext) -> Result<(), String> {
    match command.as_str() {
      AUTO_INSERT => auto_insert(),
      other => return Err(format!("unknown command {other}")),
    }
    Ok(())
  }

  fn document_opened(_: DocumentInfo) {}

  fn document_changed(_: DocumentChange) {}

  fn document_closed(_: DocumentId) {}
}

export_extension!(Xml);
