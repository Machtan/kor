//! A trie data structure to find longest contained prefixes inside a text.

use std::fmt::Debug;
use std::collections::HashMap;

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
    
    pub fn remove(&mut self, key: &str) -> Option<T> {
        let mut node = Some(&mut self.top);
        for ch in key.chars() {
            if let Some(child) = node.take().unwrap().leaves.get_mut(&ch) {
                node = Some(child);
            } else {
                return None;
            }
        }
        node.take().unwrap().value.take()
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
