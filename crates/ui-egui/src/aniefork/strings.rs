//! Interface text the fork adds, in every interface language.
//!
//! Upstream's catalogs (`i18n/*.tsv`) must all hold exactly the same entries, and upstream adds
//! to them constantly, so the fork's strings would conflict there on every merge. They are kept
//! here instead and looked up first; anything not in this table goes to upstream's catalog.
//!
//! Translated from the meaning of the English text, using the words upstream's catalogs already
//! use ("caret" is `cursor`, `каретка`, `光标`… there). Not yet read by native speakers. The
//! Turkish and Russian ones are echelon's, written for storytold/wordcraft#570.

/// English source, then its translation for every language in [`crate::i18n::LANGUAGES`].
static STRINGS: &[(&str, &[(&str, &str)])] = &[
    (
        super::options::HEADING,
        &[
            ("zh-hans", "动画"),
            ("zh-hant", "動畫"),
            ("ja", "アニメーション"),
            ("uk", "Анімація"),
            ("es", "Animación"),
            ("pt-br", "Animação"),
            ("sr", "Анимација"),
            ("sr-latn", "Animacija"),
            ("et", "Animatsioon"),
            ("tr", "Animasyon"),
            ("ru", "Анимация"),
        ],
    ),
    (
        super::options::CARET,
        &[
            ("zh-hans", "为光标添加动画（平滑移动并淡入淡出）"),
            ("zh-hant", "為游標加上動畫（平滑移動並淡入淡出）"),
            ("ja", "カーソルをアニメーション表示する（滑らかに移動してフェードする）"),
            ("uk", "Анімувати каретку (плавно рухається та згасає)"),
            ("es", "Animar el cursor (se desliza y se desvanece)"),
            ("pt-br", "Animar o ponto de inserção (desliza e esmaece)"),
            ("sr", "Анимирај курсор (клизи и постепено се гаси)"),
            ("sr-latn", "Animiraj kursor (klizi i postepeno se gasi)"),
            ("et", "Animeeri tekstikursorit (libiseb ja hajub)"),
            ("tr", "İmleci canlandır (kayarak ilerler ve solarak yanıp söner)"),
            ("ru", "Анимировать курсор (плавно перемещается и угасает)"),
        ],
    ),
    (
        super::options::CARET_HINT,
        &[
            ("zh-hans", "开启：输入或使用方向键时，光标平滑移动到新位置，闪烁时淡入淡出。关闭：光标直接跳转，闪烁时直接显示和隐藏。"),
            ("zh-hant", "開啟：輸入或使用方向鍵時，游標會平滑移動到新位置，閃爍時淡入淡出。關閉：游標直接跳到新位置，閃爍時直接顯示和隱藏。"),
            (
                "ja",
                "オン: 入力や矢印キーの操作でカーソルが新しい位置へ滑らかに移動し、点滅もフェードします。オフ: カーソルは瞬時に移動し、点滅は表示と非表示が切り替わるだけです。",
            ),
            (
                "uk",
                "Увімкнено: каретка плавно переходить на нове місце, коли ви вводите текст або користуєтеся стрілками, а її блимання згасає поступово. Вимкнено: вона перестрибує й блимає різко.",
            ),
            (
                "es",
                "Activado: el cursor se desliza hasta su nueva posición al escribir o usar las flechas, y su parpadeo se desvanece. Desactivado: salta y parpadea de golpe.",
            ),
            (
                "pt-br",
                "Ativado: o ponto de inserção desliza até a nova posição quando você digita ou usa as setas, e o piscar é suave. Desativado: ele salta e pisca de forma brusca.",
            ),
            (
                "sr",
                "Укључено: курсор клизи на ново место док куцате или користите стрелице, а његово трептање се постепено гаси. Искључено: скаче и трепће нагло.",
            ),
            (
                "sr-latn",
                "Uključeno: kursor klizi na novo mesto dok kucate ili koristite strelice, a njegovo treptanje se postepeno gasi. Isključeno: skače i trepće naglo.",
            ),
            (
                "et",
                "Sees: tekstikursor libiseb tippimisel või nooleklahvide kasutamisel uude kohta ja selle vilkumine hajub sujuvalt. Väljas: kursor hüppab ja vilgub järsult.",
            ),
            (
                "tr",
                "Açık: yazarken veya ok tuşlarını kullanırken imleç yeni yerine kayarak gider ve yanıp sönerken solar. Kapalı: imleç yeni yerine atlar, keskin biçimde yanıp söner.",
            ),
            (
                "ru",
                "Вкл.: при вводе текста и нажатии клавиш со стрелками курсор плавно перемещается на новое место, а его мигание плавно угасает. Выкл.: курсор перескакивает и резко мигает.",
            ),
        ],
    ),
];

/// `s` in the current interface language: the fork's table, then upstream's catalog, then `s`
/// itself (English).
pub fn t(s: &str) -> &str {
    #[cfg(test)]
    if crate::i18n::pseudo::on() {
        return crate::i18n::pseudo::mark(s);
    }
    let code = crate::i18n::current().code();
    match STRINGS.iter().find(|(source, _)| *source == s).and_then(|(_, translations)| translations.iter().find(|(c, _)| *c == code)) {
        Some((_, translated)) => translated,
        None => crate::i18n::t(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Lang, set_current};

    /// Every text the fork draws has one translation for each interface language: when upstream
    /// adds a language, this says what to add here.
    #[test]
    fn every_fork_text_is_translated_into_every_language() {
        let codes: Vec<&str> = Lang::all().filter(|l| *l != Lang::EN).map(Lang::code).collect();
        for text in super::super::options::TEXTS {
            let entry = STRINGS.iter().filter(|(source, _)| *source == text).collect::<Vec<_>>();
            assert_eq!(entry.len(), 1, "{text:?} needs exactly one entry");
            let translations = entry[0].1;
            for code in &codes {
                assert_eq!(translations.iter().filter(|(c, _)| c == code).count(), 1, "{text:?} needs one {code} translation");
            }
            for (code, translated) in translations {
                assert!(codes.contains(code), "{text:?}: {code} is not an interface language");
                assert!(!translated.is_empty() && translated.trim() == *translated, "{text:?} in {code}: {translated:?}");
                assert!(!translated.contains("...") && *translated != text, "{text:?} in {code}: {translated:?}");
            }
        }
        assert_eq!(STRINGS.len(), super::super::options::TEXTS.len(), "no entries nothing draws");
    }

    #[test]
    fn lookups_follow_the_interface_language_and_fall_back_to_upstreams_catalog() {
        let es = Lang::from_code("es").unwrap();
        set_current(es);
        assert_eq!(t(super::super::options::HEADING), "Animación");
        assert_eq!(t("Show rulers"), "Mostrar reglas", "not ours: upstream's catalog");
        assert_eq!(t("no such text anywhere"), "no such text anywhere");
        set_current(Lang::from_code("ja").unwrap());
        assert_eq!(t(super::super::options::HEADING), "アニメーション");
        set_current(Lang::EN);
        assert_eq!(t(super::super::options::CARET), super::super::options::CARET);
    }

    /// The interface faces shipped with the app have every letter of the Latin and Cyrillic
    /// translations (Chinese and Japanese use a system face, as upstream's catalogs do).
    #[test]
    fn bundled_interface_fonts_cover_the_translations() {
        let db = wordcraft_fonts::FontDb::with_font_dirs(Vec::new());
        for (family, style) in [("Inter", "Regular"), ("Inter", "SemiBold")] {
            let face = db.face(family, style);
            assert_eq!(face.family, family, "must use the bundled interface face");
            for (source, translations) in STRINGS {
                for (code, translated) in translations.iter().filter(|(c, _)| !matches!(*c, "ja" | "zh-hans" | "zh-hant")) {
                    for ch in translated.chars().chain(source.chars()).filter(|c| !c.is_whitespace()) {
                        assert_ne!(face.glyph_for(ch), 0, "{family} {style} lacks {ch:?} ({code})");
                    }
                }
            }
        }
    }
}
