//! Module for translating text using a dictionary.

use dict::Dict;
use wordlist::Def;

#[derive(Debug)]
pub enum TranslationPart<'def, 'src, 'defsrc: 'def> {
    /// A text part with no matching definition.
    Untranslated(&'src str),
    /// A text part that a definition was found for, along with the definition.
    Translated(&'src str, &'def Def<'defsrc>),
}

/// Attempts to replace as many words in the given text as possible with their
/// definition in the dictionary, and sends the parts to the given handler.
pub fn translate_iter<'src, 'def, 'defsrc, F>(text: &'src str, dict: &'def Dict<'defsrc>, mut handle_part: F) 
        where F: FnMut(TranslationPart<'def, 'src, 'defsrc>) {
    use self::TranslationPart::*;
    let mut untranslated_start = None;
    let mut start = 0;
    while start < text.len() {
        let rem = &text[start..];
        if let Some((prefix, def)) = dict.find_longest_match(rem) {
            if let Some(u) = untranslated_start.take() {
                handle_part(Untranslated(&text[u..start]));
            }
            handle_part(Translated(prefix, def));
            start += prefix.len();
        } else {
            untranslated_start = untranslated_start.take().or_else(|| Some(start));
            start += rem.chars().next().unwrap().len_utf8();
        }
    }
    if let Some(u) = untranslated_start.take() {
        handle_part(Untranslated(&text[u..start]));
    }
}

/// Replaces as much of text with the meanings found in the dictionary
/// as possible.
pub fn translate(text: &str, dict: &Dict) -> String {
    use self::TranslationPart::*;
    let mut translated = String::with_capacity(text.len());
    translate_iter(text, dict, |part| {
        match part {
            Untranslated(src) => {
                translated.push_str(src);
            }
            Translated(src, def) => {
                let meaning = &def.meanings[0];
                if meaning.starts_with("{") {
                    translated.push('{');
                    translated.push_str(src);
                    //translated.push_str(": ");
                    //translated.push_str(&meaning[1..]);
                } else if meaning.starts_with("<") {
                    translated.push_str(meaning);
                } else {
                    translated.push('[');
                    translated.push_str(meaning);
                    if def.hangeul.ends_with("다") {
                        //translated.push_str(": ");
                        //translated.push_str(src);
                    }
                    translated.push(']');
                }
            }
        }
    });
    translated
}
