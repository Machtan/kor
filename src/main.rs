#![feature(unicode)]

extern crate std_unicode;
extern crate regex;
extern crate unicode_normalization;
#[macro_use]
extern crate lazy_static;
extern crate hangeul2;
extern crate argonaut;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use std_unicode::str::UnicodeStr;
use std::fmt::Debug;
use std::collections::HashMap;
use std::borrow::Cow;
use std::rc::Rc;
use hangeul2::{is_hangeul, Initial, Block};
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::env;
use argonaut::{ArgDef, parse, ParseError, help_arg, version_arg};
use std::process;
use std::error::Error;

//const SAMPLE: &str = include_str!("../resources/ch1_sample.txt");
//const WORD_LIST: &str = include_str!("../resources/ark.wl.txt");

lazy_static! {
    pub static ref RE_DEF: Regex = {
        Regex::new(r"^(.+?)\s*(?:[\(（]\s*(.+?)[\)）]\s*)?$").expect("RE_DEF")
    };
    pub static ref RE_HANGEUL: Regex = {
        Regex::new(r"^[\-~-]?\s*(.*?)\s*$").expect("RE_HANGEUL")
    };
}

#[derive(Debug, Clone)]
pub struct Def<'src> {
    hangeul: Cow<'src, str>,
    aliases: Vec<Cow<'src, str>>,
    hanja: Option<Cow<'src, str>>,
    meanings: Vec<Cow<'src, str>>,
}

pub fn clean_hangeul(hangeul: &str) -> &str {
    let caps = RE_HANGEUL.captures(hangeul)
        .expect(&format!("Invalid hangeul: {:?}", hangeul));
    caps.get(1).unwrap().as_str()
}

pub fn read_definition<'src>(line: &'src str) -> Def<'src> {
    let caps = RE_DEF.captures(line)
        .expect(&format!("Invalid definition line: {:?}", line));
    let hangeul_blocks = caps.get(1).unwrap().as_str();
    let mut parts = hangeul_blocks.split("|").map(|s| clean_hangeul(s).into());
    let hangeul = parts.next().unwrap();
    let aliases = parts.collect::<Vec<_>>();
    let hanja = caps.get(2).map(|m| m.as_str().into());
    Def { hangeul, aliases, hanja, meanings: Vec::new() }
}

pub fn read_meaning<'src>(line: &'src str) -> Cow<'src, str> {
    line.trim().into()
}

pub fn read_definitions_with<'src, F: FnMut(Def<'src>)>(text: &'src str, mut add_def: F) {
    let mut def: Option<Def<'src>> = None;
    for (i, line) in text.lines().enumerate() {
        if line.starts_with("#") || line.is_whitespace() {
            continue;
        }
        if ! line.starts_with("  ") {
            if let Some(def) = def.take() {
                add_def(def);
            }
            def = Some(read_definition(line));
        } else {
            if let Some(ref mut def) = def {
                def.meanings.push(read_meaning(line));
            } else {
                panic!("Line {}: Meaning found without definition", i);
            }
        }
    }
    if let Some(def) = def {
        add_def(def);
    }
}

pub fn read_definitions<'src>(text: &'src str) -> Vec<Def<'src>> {
    let mut defs = Vec::new();
    read_definitions_with(text, |def| defs.push(def));
    defs
}

#[derive(Debug, Clone)]
pub struct Trie<T: Debug> {
    top: TrieNode<T>
}
impl<T: Debug> Trie<T> {
    pub fn new() -> Trie<T> {
        Trie { top: TrieNode::new() }
    }
    
    pub fn find_longest_match<'n, 'k>(&'n self, key: &'k str) -> Option<(&'k str, &'n T)> {
        self.top.find_longest_match(key, 0)
    }
    
    pub fn find_shortest_match<'n, 'k>(&'n self, key: &'k str) -> Option<(&'k str, &'n T)> {
        self.top.find_closest_value(key).map(|(prefix, node)| {
            (prefix, node.value.as_ref().unwrap())
        }) 
    }
    
    pub fn insert(&mut self, key: &str, value: T) {
        self.top.insert(key, value);
    }
}

#[derive(Debug, Clone)]
struct TrieNode<T: Debug> {
    value: Option<T>,
    leaves: HashMap<char, Box<TrieNode<T>>>
}
impl<T: Debug> TrieNode<T> {
    fn new() -> TrieNode<T> {
        TrieNode { value: None, leaves: HashMap::new() }
    }
    
    fn find_closest_value<'n, 'k>(&'n self, key: &'k str) -> Option<(&'k str, &'n TrieNode<T>)> {
        let mut node = &*self;
        for (i, ch) in key.char_indices() {
            if let Some(child) = node.leaves.get(&ch) {
                if child.value.is_some() {
                    let prefix = &key[.. i + ch.len_utf8()];
                    return Some((prefix, child));
                }
                node = child;
            } else {
                break;
            }
        }
        None
    }
    
    fn find_longest_match<'n, 'k>(&'n self, key: &'k str, start: usize) -> Option<(&'k str, &'n T)> {
        let rel_key = &key[start..];
        if let Some((prefix, node)) = self.find_closest_value(rel_key) {
            let new_start = start + prefix.len();
            node.find_longest_match(key, new_start)
        } 
        else if start == 0 {
            None
        } 
        else {
            if let Some(ref value) = self.value {
                Some((&key[..start], value))
            } else {
                None
            }
        }
    }
    
    fn insert(&mut self, key: &str, value: T) {
        if key.len() == 0 {
            self.value = Some(value);
        } else {
            let ch = key.chars().nth(0).unwrap();
            let rem = &key[ch.len_utf8()..];
            self.leaves.entry(ch).or_insert_with(|| Box::new(TrieNode::new())).insert(rem, value);
        }
    }
}

#[derive(Debug)]
pub enum TranslationPart<'def, 'src, 'defsrc: 'def> {
    Untranslated(&'src str),
    Translated(&'src str, &'def Def<'defsrc>),
}

pub type Dict<'defsrc> = Trie<Rc<Def<'defsrc>>>;

pub fn translate_split<'src, 'def, 'defsrc, F>(text: &'src str, dict: &'def Dict<'defsrc>, mut send_part: F) 
        where F: FnMut(TranslationPart<'def, 'src, 'defsrc>) {
    use TranslationPart::*;
    let mut untranslated_start = None;
    let mut start = 0;
    while start < text.len() {
        let rem = &text[start..];
        if let Some((prefix, def)) = dict.find_longest_match(rem) {
            if let Some(u) = untranslated_start.take() {
                send_part(Untranslated(&text[u..start]));
            }
            send_part(Translated(prefix, def));
            start += prefix.len();
        } else {
            untranslated_start = untranslated_start.take().or_else(|| Some(start));
            start += rem.chars().next().unwrap().len_utf8();
        }
    }
    if let Some(u) = untranslated_start.take() {
        send_part(Untranslated(&text[u..start]));
    }
}

pub fn translate(text: &str, dict: &Dict) -> String {
    use TranslationPart::*;
    let mut translated = String::with_capacity(text.len());
    translate_split(text, dict, |part| {
        match part {
            Untranslated(src) => {
                translated.push_str(src);
            }
            Translated(src, def) => {
                let meaning = &def.meanings[0];
                if meaning.starts_with("{") {
                    translated.push('{');
                    translated.push_str(src);
                    translated.push_str(": ");
                    translated.push_str(&meaning[1..]);
                } else if meaning.starts_with("<") {
                    translated.push_str(meaning);
                } else {
                    translated.push('[');
                    translated.push_str(meaning);
                    if def.hangeul.ends_with("다") {
                        translated.push_str(": ");
                        translated.push_str(src);
                    }
                    translated.push(']');
                }
            }
        }
    });
    translated
}

pub fn with_simple_conjugations<F: FnMut(&str)>(stem: &str, mut send_conj: F) {
    use hangeul2::Vowel::*;
    use hangeul2::Final::*;
    send_conj(stem);
    let last = stem.chars().last().unwrap();
    if ! is_hangeul(last) {
        println!("LAST IS NOT HANGEUL!: {:?}", stem);
        return;
    }
    let prefix = {
        let mut chars = stem.chars();
        chars.next_back();
        chars.collect::<String>()
    };
    macro_rules! push_with_last {
        ($block:expr) => {{
            let mut text = prefix.clone();
            text.push($block.into());
            send_conj(&text);
        }}
    }

    let block = Block::new(last).unwrap();
    match block.final_ {
        Empty => {
            push_with_last!(block.with_final(N)); // past/descr
            push_with_last!(block.with_final(L)); // fut
            push_with_last!(block.with_final(B)); // (sy)bnida ending
            
            match block.vowel {
                A => {
                    push_with_last!(block.with_vowel(Ae));
                    push_with_last!(block.with_vowel(Ae).with_final(Ss));
                }
                I => {
                    push_with_last!(block.with_vowel(Yeo));
                    push_with_last!(block.with_vowel(Yeo).with_final(Ss));
                }
                Y => {
                    push_with_last!(block.with_vowel(Eo));
                    push_with_last!(block.with_vowel(Eo).with_final(Ss));
                }
                _ => {
                    push_with_last!(block.with_final(L));
                    push_with_last!(block.with_final(Ss));
                }
            }
        }
        L => {
            push_with_last!(block.with_final(Empty));
            push_with_last!(block.with_final(N));
        }
        B => {
            let mut u_end = prefix.clone();
            u_end.push(block.with_final(Empty).into());
            
            let mut un_end = u_end.clone();
            let mut ul_end = u_end.clone();
            let mut weo_end = u_end.clone();
            let mut weoss_end = u_end.clone();
            
            u_end.push(Block::from_parts(Initial::Ieung, U, Empty).into());
            send_conj(&u_end);
            
            un_end.push(Block::from_parts(Initial::Ieung, U, N).into());
            send_conj(&un_end);
            
            ul_end.push(Block::from_parts(Initial::Ieung, U, L).into());
            send_conj(&ul_end);
            
            weo_end.push(Block::from_parts(Initial::Ieung, Weo, Empty).into());
            send_conj(&weo_end);
            
            weoss_end.push(Block::from_parts(Initial::Ieung, Weo, Ss).into());
            send_conj(&weoss_end);
        }
        _ => {}
    }
}

fn add_defs_to_dict<'src>(dict: &mut Dict<'src>, defs: Vec<Def<'src>>) {
    for def in defs {
        let def = Rc::new(def);
        for key in Some(def.hangeul.clone()).iter().chain(def.aliases.iter()) {
            if (&key).ends_with("하다") {
                dict.insert(&key[..key.len() - "하다".len()], def.clone());
            } else if (&key).ends_with("다") {
                with_simple_conjugations(&key[..key.len() - "다".len()], |conj| {
                    dict.insert(conj, def.clone());
                });
            } else {
                dict.insert(&key, def.clone());
            }
        }
    }
}

#[allow(unused)]
fn old_main() {
    let mut word_list_src = String::new();
    File::open("resources/ark.wl.txt").expect("! open word list")
        .read_to_string(&mut word_list_src).expect("! read word list");
    
    let def_text = word_list_src.nfc().collect::<String>();
    let defs = read_definitions(&def_text);
    
    let test_line = "유치하다 (幼稚- | 幼穉-)  ";
    assert!(RE_DEF.is_match(test_line));
    let caps = RE_DEF.captures(test_line).unwrap();
    println!("Full match: {:?}", caps.get(0).unwrap().as_str());
    println!("Hangeul: {}", caps.get(1).unwrap().as_str());
    println!("Hanja:   {}", caps.get(2).unwrap().as_str());
    
    let test_line_2 = "유치하다";
    let caps = RE_DEF.captures(test_line_2).unwrap();
    println!("Full match: {:?}", caps.get(0).unwrap().as_str());
    println!("Hangeul: {}", caps.get(1).unwrap().as_str());
    println!("Hanja:   {}", caps.get(2).map(|m| m.as_str()).unwrap_or(""));
    
    println!("Definitions:");
    for def in &defs {
        println!("  {:?}", def);
    }
    
    let mut trie = Trie::new();
    add_defs_to_dict(&mut trie, defs);
    println!("Trie:");
    println!("{:?}", trie);
    
    let key = "유치하다하다";
    println!("Closest node for {:?}: {:?}", key, trie.find_shortest_match(key));
    let key2 = "드문드문";
    println!("Closest node for  {:?}: {:?}", key2, trie.find_shortest_match(key2));
    println!("Longest match for {:?}: {:?}", key2, trie.find_longest_match(key2));
    
    let mut sample = String::new();
    File::open("resources/ch1_sample.txt").expect("! open sample")
        .read_to_string(&mut sample).expect("! read sample");
    let translated = translate(&sample, &trie);
    println!("Translated:");
    println!("{}", translated);
    
}

fn main() {
    // Properly set exit codes after the program has cleaned up.
    if let Some(exit_code) = argonaut_main() {
        process::exit(exit_code);
    }
}

fn argonaut_main() -> Option<i32> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    
    let mut word_list_files: Vec<String> = Vec::new();
    let mut document_path = String::new();
    
    const DESC: &str = "
        Makes a very rough translation of a document in Korean, using a set
        of provided word lists to substitute words with their definitions.
    ";
    
    match parse("kor", &args, vec![
        ArgDef::positional("document", &mut document_path)
            .help("A document to translate"),
        
        ArgDef::collect("word-list", &mut word_list_files)
            // Adds '-i' as an alias for this option.
            .short("w")
            .param("file")
            .help("Word lists to read definitions from."),
        
        help_arg(DESC).short("h"),
        version_arg(),
    ]) {
        Ok(_optional_error_code) => {},
        Err(ParseError::Interrupted(_)) => {
            return None;
        },
        Err(_) => {
            return Some(1);
        }
    };
    
    macro_rules! open_and_read_to_string {
        ($path:expr, $string:expr) => {{
            let mut file = match File::open($path) {
                Ok(f) => f,
                Err(e) => {
                    let _ = write!(io::stderr(), "Could not open file {:?}: {:?}", $path, e.description());
                    return Some(2);
                }
            };
            if let Err(e) = file.read_to_string(&mut $string) {
                let _ = write!(io::stderr(), "Could not read file {:?}: {:?}", $path, e.description());
                return Some(3);
            }
        };}
    }
    
    let mut def_sources = Vec::new();
    for path in &word_list_files {
        let mut source = String::new();
        open_and_read_to_string!(path, &mut source);
        def_sources.push(source);
    }
    
    let mut defs = Vec::new();
    for source in &def_sources {
        read_definitions_with(source, |def| {
            defs.push(def);
        });
    }
    
    let mut dict = Trie::new();
    add_defs_to_dict(&mut dict, defs);
    
    let mut text = String::new();
    open_and_read_to_string!(&document_path, &mut text);
    
    let translated = translate(&text, &dict);
    println!("{}", translated);
    
    None
}

