//! Module for working with dictionaries.

use std::rc::Rc;
use trie::Trie;
use wordlist::Def;
use hangeul2::{Block, Initial, is_hangeul};

/// The dictionary type used for translations.
pub struct Dict<'defsrc> {
    inner: Trie<Rc<Def<'defsrc>>>,
}

impl<'defsrc> Dict<'defsrc> {
    /// Creates a new dictionary.
    pub fn new() -> Dict<'defsrc> {
        Dict {
            inner: Trie::new(),
        }
    }
    
    /// Finds the definition matching as many characters of the key as possible.
    pub fn find_longest_match<'n, 'k>(&'n self, key: &'k str) -> Option<(&'k str, &'n Rc<Def<'defsrc>>)> {
        self.inner.find_longest_match(key)
    }
    
    /// Finds the definition matching as few characters of the key as possible.
    pub fn find_shortest_match<'n, 'k>(&'n self, key: &'k str) -> Option<(&'k str, &'n Rc<Def<'defsrc>>)> {
        self.inner.find_shortest_match(key)
    }
    
    /// Inserts a definition.
    pub fn insert(&mut self, key: &str, value: Rc<Def<'defsrc>>) {
        self.inner.insert(key, value)
    }
    
    /// Removes a definition, if any.
    pub fn remove(&mut self, key: &str) -> Option<Rc<Def<'defsrc>>> {
        self.inner.remove(key)
    }
    
    /// Adds the given definitions to the dictionary.
    /// Newer definitions of a word replace older ones.
    pub fn add_definitions(&mut self, defs: Vec<Def<'defsrc>>) {
        for def in defs {
            let def = Rc::new(def);
            for key in Some(def.hangeul.clone()).iter().chain(def.aliases.iter()) {
                if (&key).ends_with("하다") {
                    self.inner.insert(&key[..key.len() - "하다".len()], def.clone());
                } else if (&key).ends_with("다") {
                    simple_conjugations_iter(&key[..key.len() - "다".len()], |conj| {
                        self.inner.insert(conj, def.clone());
                    });
                } else {
                    self.inner.insert(&key, def.clone());
                }
            }
        }
    }
}

/// Conjugates the given stem to a list of common forms in Korean.
/// The conjugated versions are then 'sent' to the handler. 
fn simple_conjugations_iter<F: FnMut(&str)>(stem: &str, mut handle_conj: F) {
    use hangeul2::Vowel::*;
    use hangeul2::Final::*;
    handle_conj(stem);
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
            handle_conj(&text);
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
            handle_conj(&u_end);
            
            un_end.push(Block::from_parts(Initial::Ieung, U, N).into());
            handle_conj(&un_end);
            
            ul_end.push(Block::from_parts(Initial::Ieung, U, L).into());
            handle_conj(&ul_end);
            
            weo_end.push(Block::from_parts(Initial::Ieung, Weo, Empty).into());
            handle_conj(&weo_end);
            
            weoss_end.push(Block::from_parts(Initial::Ieung, Weo, Ss).into());
            handle_conj(&weoss_end);
        }
        _ => {}
    }
}
