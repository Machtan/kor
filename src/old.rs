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