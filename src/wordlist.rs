//! Module for working with word list files and data.

use std::borrow::Cow;
use regex::Regex;
use common::*;
use std_unicode::str::UnicodeStr;

lazy_static! {
    pub static ref RE_DEF: Regex = {
        Regex::new(r"^(.+?)\s*(?:[\(（]\s*(.+?)[\)）]\s*)?$").expect("RE_DEF")
    };
    pub static ref RE_HANGEUL: Regex = {
        Regex::new(r"^[\-~-]?\s*(.*?)\s*$").expect("RE_HANGEUL")
    };
}

/// A definition entry.
#[derive(Debug, Clone)]
pub struct Def<'src> {
    pub hangeul: Cow<'src, str>,
    pub aliases: Vec<Cow<'src, str>>,
    pub hanja: Option<Cow<'src, str>>,
    pub meanings: Vec<Cow<'src, str>>,
}

/// Cleans the hangeul part of a word list definition.
fn clean_hangeul(hangeul: &str) -> &str {
    let caps = if let Some(caps) = RE_HANGEUL.captures(hangeul) {
        caps
    } else {
        warn!("Could not parse hangeul: {:?}", hangeul);
        return hangeul;
    };
    caps.get(1).unwrap().as_str()
}

/// Reads the first line of a word list definition.
fn read_definition<'src>(line: &'src str, lineno: usize) -> Option<Def<'src>> {
    let caps = if let Some(caps) = RE_DEF.captures(line) {
        caps
    } else {
        warn!("Line {}: Invalid definition: {:?}", lineno, line);
        return None;
    };
    let hangeul_blocks = caps.get(1).unwrap().as_str();
    let mut parts = hangeul_blocks.split("|").map(|s| clean_hangeul(s).into());
    let hangeul = parts.next().unwrap();
    let aliases = parts.collect::<Vec<_>>();
    let hanja = caps.get(2).map(|m| m.as_str().into());
    Some(Def { hangeul, aliases, hanja, meanings: Vec::new() })
}

/// Reads a meaning from a line in a word-list.
fn read_meaning<'src>(line: &'src str) -> Cow<'src, str> {
    line.trim().into()
}

/// Reads word definitions from a text and calls 'add_def' for each loaded 
/// definition.
pub fn read_definitions_iter<'src, F: FnMut(Def<'src>)>(text: &'src str, mut add_def: F) {
    let mut def: Option<Def<'src>> = None;
    for (i, line) in text.lines().enumerate() {
        if line.starts_with("#") 
        || line.starts_with("  #") 
        || line.is_whitespace() {
            continue;
        
        } else if ! line.starts_with("  ") {
            if let Some(def) = def.take() {
                add_def(def);
            }
            def = read_definition(line, i+1);
        
        } else {
            if let Some(ref mut def) = def {
                def.meanings.push(read_meaning(line));
            } else {
                warn!("Line {}: Meaning found without definition", i+1);
            }
        }
    }
    if let Some(def) = def {
        add_def(def);
    }
}

/// Reads the word definitions in the given text and returns them in a vector.
pub fn read_definitions<'src>(text: &'src str) -> Vec<Def<'src>> {
    let mut defs = Vec::new();
    read_definitions_iter(text, |def| defs.push(def));
    defs
}
