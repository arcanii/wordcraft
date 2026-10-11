//! Front-end code private to the aniefork fork of WordCraft.
//!
//! Everything the fork adds to the UI lives under this module, so merges from upstream
//! (`storytold/wordcraft`) only ever meet the short hooks that call into it. See
//! `AGENTS-aniefork.md` at the repository root.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::WordApp;

pub mod caret;
pub mod options;
pub mod strings;

/// The fork's settings. They are saved with the rest of the interface settings (`ui.json`, under
/// `aniefork`), so a build without the fork ignores them and settings from before them still load.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Prefs {
    /// File ▸ Options ▸ Animation: the caret glides and fades as Word's does (on, the default), or
    /// jumps and blinks on and off as upstream's does (off).
    pub caret_animation: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { caret_animation: true }
    }
}

/// The fork's commands in the UI layer (`None`: not one of them).
pub(crate) fn ui_command(app: &mut WordApp, id: &str, p: &Value) -> Option<Result<Value, String>> {
    match id {
        // File ▸ Options ▸ Animation: `value` true or false turns the caret's animation on or
        // off; no value reads it.
        "ui.caretAnimation" => {
            match p.get("value") {
                None => {}
                Some(Value::Bool(on)) => app.ui.aniefork.caret_animation = *on,
                Some(_) => return Some(Err("`value` must be true or false".into())),
            }
            Some(Ok(json!({"caretAnimation": app.ui.aniefork.caret_animation})))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;

    use super::*;

    fn app() -> WordApp {
        WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Default::default())
    }

    #[test]
    fn the_caret_animation_setting_is_a_ui_command_and_persists() {
        let mut a = app();
        assert!(a.ui.aniefork.caret_animation, "on unless turned off");
        assert_eq!(a.run("ui.caretAnimation", json!({})).unwrap(), json!({"caretAnimation": true}), "no value reads it");
        assert_eq!(a.run("ui.caretAnimation", json!({"value": false})).unwrap(), json!({"caretAnimation": false}));
        assert!(!a.ui.aniefork.caret_animation);
        // Junk leaves it alone.
        for junk in [json!({"value": "yes"}), json!({"value": 1}), json!({"value": null}), json!({"value": [true]})] {
            assert!(a.run("ui.caretAnimation", junk).is_err());
        }
        assert!(a.run("ui.caretAnimation", json!("junk")).is_ok());
        assert!(!a.ui.aniefork.caret_animation);
        // It is saved with the interface settings and comes back.
        let saved = serde_json::to_string(&a.prefs()).unwrap();
        assert!(saved.contains(r#""aniefork":{"caretAnimation":false}"#), "{saved}");
        let restored: crate::UiState = serde_json::from_str(&saved).unwrap();
        assert!(!restored.aniefork.caret_animation);
        // Settings saved before the option existed, or damaged, leave it on.
        for old in [r#"{"tab":"Home"}"#, r#"{"aniefork":{}}"#, r#"{"aniefork":{"somethingElse":3}}"#] {
            assert!(serde_json::from_str::<crate::UiState>(old).unwrap().aniefork.caret_animation, "{old}");
        }
        assert_eq!(a.run("ui.caretAnimation", json!({"value": true})).unwrap()["caretAnimation"], true);
    }

    /// File ▸ Options has an Animation section whose checkbox is the setting.
    #[test]
    fn options_has_an_animation_section() {
        let mut a = app();
        a.run("ui.backstage", json!({"value": true, "page": "options"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        assert!(h.query_by_label(options::HEADING).is_some(), "the section is there");
        assert!(h.query_by_label("Display").is_some() && h.query_by_label("Agents").is_some(), "between upstream's own sections");
        h.get_by_label(options::CARET).click();
        h.run_steps(4);
        assert!(!h.state().ui.aniefork.caret_animation, "the checkbox turns it off");
        h.get_by_label(options::CARET).click();
        h.run_steps(4);
        assert!(h.state().ui.aniefork.caret_animation, "and on again");
        // In another interface language the section is translated.
        h.state_mut().run("ui.language", json!({"value": "es"})).unwrap();
        h.run_steps(4);
        assert!(h.query_by_label("Animación").is_some() && h.query_by_label(options::HEADING).is_none());
    }
}
