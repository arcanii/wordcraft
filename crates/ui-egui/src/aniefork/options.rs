//! File ▸ Options ▸ Animation: the fork's section of the Options page.

use egui::Ui;
use serde_json::json;

use super::strings::t;
use crate::WordApp;
use crate::theme::semibold;

pub(crate) const HEADING: &str = "Animation";
pub(crate) const CARET: &str = "Animate the caret (it glides and fades)";
pub(crate) const CARET_HINT: &str =
    "On: the caret glides to its new place when you type or use the arrow keys, and its blink fades. Off: it jumps and blinks on and off.";
/// Every text this section draws (the tests check each one is translated).
#[cfg(test)]
pub(crate) const TEXTS: [&str; 3] = [HEADING, CARET, CARET_HINT];

/// The section, drawn between upstream's Display and Agents sections.
pub(crate) fn section(app: &mut WordApp, ui: &mut Ui) {
    ui.label(egui::RichText::new(t(HEADING)).font(semibold(15.0)));
    let mut on = app.ui.aniefork.caret_animation;
    if ui.checkbox(&mut on, t(CARET)).on_hover_text(t(CARET_HINT)).changed() {
        let _ = app.run("ui.caretAnimation", json!({"value": on}));
    }
    ui.add_space(10.0);
}
