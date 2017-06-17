#![feature(unicode)]

extern crate argonaut;
extern crate kor;
extern crate std_unicode;

use std_unicode::str::UnicodeStr;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::env;
use argonaut::{ArgDef, parse, ParseError, help_arg, version_arg};
use std::process;
use std::error::Error;
use kor::{Dict, translate, read_definitions_iter};

//const SAMPLE: &str = include_str!("../resources/ch1_sample.txt");
//const WORD_LIST: &str = include_str!("../resources/ark.wl.txt");

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

#[derive(Debug)]
pub enum TranslationMode {
    Normal,
    LineByLineWithSpace,
    Retranslate,
}

pub const AUTO_PREFIX:   &str = r"->";
pub const MANUAL_PREFIX: &str = r"-|";

// NOTE: Should this be in main or in translate?
// -> I should probably split the loading/doing parts, and make it output
// to something that isn't stdout :p.
// Take a 'target' that can be both stdout and a string? or just a string?
fn cmd_translate(document_path: &str, word_list_files: &Vec<String>, 
        exclusion_files: &Vec<String>, mode: TranslationMode) 
        -> Option<i32> {
    use self::TranslationMode::*;
    
    let mut def_sources = Vec::new();
    for path in word_list_files {
        let mut source = String::new();
        open_and_read_to_string!(path, &mut source);
        def_sources.push(source);
    }
    
    let mut defs = Vec::new();
    for source in &def_sources {
        read_definitions_iter(source, |def| {
            defs.push(def);
        });
    }
    
    
    let mut dict = Dict::new();
    dict.add_definitions(defs);
    
    for path in exclusion_files {
        let mut source = String::new();
        open_and_read_to_string!(path, &mut source);
        for line in source.lines() {
            if line.starts_with("#") || line.is_whitespace() {
                continue;
            } else {
                dict.remove(line.trim());
            }
        }
    }
    
    let mut text = String::new();
    open_and_read_to_string!(&document_path, &mut text);
    
    match mode {
        Normal => {
            let translated = translate(&text, &dict);
            println!("{}", translated);
        }
        LineByLineWithSpace => {
            for line in text.lines() {
                if line.is_whitespace() {
                    println!("{}", line);
                    continue;
                }
                let translated = translate(line, &dict);
                println!("{}", line);
                if translated != line {
                    println!("{} {}", AUTO_PREFIX, translated);
                }
                println!("{} ", MANUAL_PREFIX);
                println!("{} ", MANUAL_PREFIX);
                println!("{} ", MANUAL_PREFIX);
            }
        }
        Retranslate => {
            for line in text.lines() {
                if line.is_whitespace() {
                    println!("{}", line);
                
                } else if line.starts_with(MANUAL_PREFIX) {
                    println!("{}", line);
                
                } else if line.starts_with(AUTO_PREFIX) {
                    // Remove the automatically translated lines
                
                } else {
                    let translated = translate(line, &dict);
                    println!("{}", line);
                    if translated != line {
                        println!("{} {}", AUTO_PREFIX, translated);
                    }
                }
            }
        }
    }
    
    None
}

fn cmd_clean(document_path: &str) -> Option<i32> {
    let mut text = String::new();
    open_and_read_to_string!(&document_path, &mut text);
    
    for line in text.lines() {
        if line.is_whitespace() {
            println!("{}", line.trim());
        } else if line.starts_with(MANUAL_PREFIX) {
            let rem = (&line[MANUAL_PREFIX.len()..]).trim();
            if rem.is_whitespace() {
                // Indicate that something wasn't translated
                println!("{} ", MANUAL_PREFIX);
            } else {
                println!("{}", rem);
            }
        }
    }
    
    None
}

fn main() {
    // Properly set exit codes after the program has cleaned up.
    if let Some(exit_code) = argonaut_main() {
        process::exit(exit_code);
    }
}

fn argonaut_main() -> Option<i32> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    
    const DESC: &str = "
        Utility to aid with the translation of Korean text documents.
    ";
    
    match parse("kor", &args, vec![
        // TRANSLATE
        ArgDef::subcommand("translate", |name, args| {
            const DESC: &str = "
                Makes a very rough translation of a document in Korean, using a set
                of provided word lists to substitute words with their definitions.
            ";
            
            let mut word_list_files: Vec<String> = Vec::new();
            let mut document_path = String::new();
            let mut exclusion_files: Vec<String> = Vec::new();
            let mut use_line_mode = false;
            let mut retranslate_instead = false;
            
            parse(name, args, vec![
                  ArgDef::positional("document", &mut document_path)
                    .help("A document to translate")
        
                , ArgDef::collect("word-list", &mut word_list_files)
                    .short("w")
                    .param("file")
                    .help("Word lists to read definitions from.")
                ,
                 ArgDef::collect("exclusion-rules", &mut exclusion_files)
                    .short("x")
                    .param("file")
                    .help("Files with one word per line to exclude from the automatic translation")
                
                , ArgDef::flag("retranslate", &mut retranslate_instead)
                    .short("r")
                    .help("Retranslates the file, keeping existing user-translated lines")
                
                , ArgDef::flag("use-line-mode", &mut use_line_mode)
                    .short("l")
                    .help("
                        Keep existing lines, and place a translated and blank line under
                        each line of the source text.
                    ")
                
                , help_arg(DESC).short("h")
            ])?;
            
            let mut mode = TranslationMode::Normal;
            if use_line_mode {
                mode = TranslationMode::LineByLineWithSpace;
            }
            if retranslate_instead {
                mode = TranslationMode::Retranslate;
            }
            
            let res = cmd_translate(&document_path, &word_list_files, &exclusion_files, mode);
            Ok(res)
        })
        
        , ArgDef::subcommand("clean", |name, args| {
            const DESC: &str = "
            
            ";
            
            let mut document_path = String::new();
            
            parse(name, args, vec![
                  ArgDef::positional("document", &mut document_path)
                    .help("A document to translate")
                  
                , help_arg(DESC).short("h")
            ])?;
            
            let res = cmd_clean(&document_path);
            Ok(res)
        })
        
        , help_arg(DESC).short("h")
        , version_arg()
    ]) {
        Ok(_optional_error_code) => {},
        Err(ParseError::Interrupted(_)) => {
            return None;
        },
        Err(_) => {
            return Some(1);
        }
    };
    
    None
}

