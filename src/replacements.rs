use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
struct Character {
    jp_names: Vec<String>,
    en_names: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Rule {
    key: String,
    name: String,
    split: bool,
    honorifics: Option<(bool, bool)>,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Replacements {
    rules: Vec<Rule>,
    honorifics: HashMap<String, String>,
    contents: serde_json::Value,
}

pub fn replace_from_json(replacement_json: PathBuf, input_file: PathBuf, output_file: PathBuf) {
    let replace_json_contents =
        fs::read_to_string(replacement_json).expect("Cannot read Replacement File.");
    let rep_cont: Replacements =
        serde_json::from_str(&replace_json_contents).expect("Can't parse Json");
    verify_rules(&rep_cont);

    let input_file_contents = fs::read_to_string(input_file).expect("Cannot read input file.");
    let output_file_contents = replace_string(&rep_cont, input_file_contents);
    fs::write(output_file, output_file_contents).expect("Cannot write to output file.");
}

fn verify_rules(rep: &Replacements) {
    for rule in rep.rules.iter() {
        rep.contents
            .get(rule.key.clone())
            .expect(&format!("No Key: {} found in replacements json", rule.key));
    }
}

fn replace_string(rep: &Replacements, contents: String) -> String {
    let mut output: String = contents;
    for rule in rep.rules.iter() {
        println!(
            "{} [{}]: {}",
            format!("* {}", rule.name).bold().green(),
            rule.key,
            rule.description.as_ref().unwrap_or(&String::from(""))
        );
        if rule.honorifics.is_none() {
            // not a name replacement
            assert!(rule.split == false, "Cannot split non-name replacements.");
            let rep_dict: HashMap<String, String> =
                serde_json::from_str(&rep.contents.get(&rule.key).unwrap().to_string())
                    .expect(&format!("Cannot parse contents of key: {}.", &rule.key));
            output = replace_hashmap(output, &rep_dict);
        } else {
            if rule.split {
                let (first_name, last_name) = rule.honorifics.unwrap();
                let rep_dict: HashMap<String, Vec<String>> =
                    serde_json::from_str(&rep.contents.get(&rule.key).unwrap().to_string())
                        .expect(&format!("Cannot parse contents of key: {}.", &rule.key));
                output = replace_names(output, &rep_dict, &rep.honorifics, first_name, last_name);
            } else {
                let (first_name, _) = rule.honorifics.unwrap();
                let rep_dict: HashMap<String, String> =
                    serde_json::from_str(&rep.contents.get(&rule.key).unwrap().to_string())
                        .expect(&format!("Cannot parse contents of key: {}.", &rule.key));
                let mut hon_dict = HashMap::<String, String>::new();

                for (en_name, jp_name) in rep_dict.iter() {
                    for (hon_jp, hon_en) in rep.honorifics.iter() {
                        hon_dict.insert(
                            format!("{}{}", jp_name, hon_jp),
                            format!("{}-{}", en_name, hon_en),
                        );
                    }
                }
                output = replace_hashmap(output, &hon_dict);
                if !first_name {
                    output = replace_hashmap(output, &rep_dict);
                }
            }
        }
    }
    return output;
}

fn replace_names(
    contents: String,
    rep_dict: &HashMap<String, Vec<String>>,
    honorifics: &HashMap<String, String>,
    first_name: bool,
    last_name: bool,
) -> String {
    let mut output: String = contents;
    for (en_name, jp_names) in rep_dict.iter() {
        for (hon_jp, hon_en) in honorifics.iter() {
            output = replace_single(
                output,
                &format!("{}{}", jp_names.join(""), hon_jp),
                &format!("{}-{}", en_name, hon_en),
            );
        }
        output = replace_single(output, &jp_names.join("・"), &en_name.to_string());
        output = replace_single(output, &jp_names.join(""), &en_name.to_string());
    }
    for (en_name, jp_names) in rep_dict.iter() {
        let en_names: Vec<&str> = en_name.split(" ").collect();
        assert!(
            en_names.len() == jp_names.len(),
            "{}: {:?} {:?}",
            "English and Japanese names are different length",
            en_names,
            jp_names
        );
        for i in 0..en_names.len() {
            for (hon_jp, hon_en) in honorifics.iter() {
                output = replace_single(
                    output,
                    &format!("{}{}", jp_names[i], hon_jp),
                    &format!("{}-{}", en_names[i], hon_en),
                );
            }
            // len > 3 to filter names with single kanji (len is 3 for
            // one kanji), so that they won't be replaced in random
            // words.  Those names will only be replaced when they
            // occur with honorifics. Very hard to tell it's a name
            // from the code otherwise.
            if i == 0 && !first_name && jp_names[i].len() > 3 {
                output = replace_single(
                    output,
                    &format!("{}", jp_names[i]),
                    &format!("{}", en_names[i]),
                );
            } else if i == en_names.len() - 1 && !last_name && jp_names[i].len() > 3 {
                output = replace_single(
                    output,
                    &format!("{}", jp_names[i]),
                    &format!("{}", en_names[i]),
                );
            }
        }
    }
    return output;
}

fn replace_hashmap(contents: String, map: &HashMap<String, String>) -> String {
    let mut output: String = contents;
    for (find, replace) in map.iter() {
        output = replace_single(output, find, replace);
    }
    return output;
}

fn replace_single(contents: String, find: &String, replace: &String) -> String {
    let mut output: String = contents;
    let count = output.matches(find).count();
    if count > 0 {
        output = output.replace(find, &replace);
        println!("{} → {} ({})", find, replace, format!("{}", count).bold());
    }
    return output;
}
