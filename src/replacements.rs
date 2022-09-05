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

pub fn replace_from_json(
    threshold: usize,
    replacement_json: PathBuf,
    input_file: PathBuf,
    output_file: PathBuf,
) -> Result<(), String> {
    let replace_json_contents = match fs::read_to_string(replacement_json) {
        Ok(c) => c,
        Err(e) => return Err(format!("{}\n{:?}", "Cannot read Replacement File.", e,)),
    };
    let rep_cont: Replacements = match serde_json::from_str(&replace_json_contents) {
        Ok(j) => j,
        Err(e) => return Err(format!("{}\n{:?}", "Can't parse Json", e)),
    };

    match verify_rules(&rep_cont) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };

    let input_file_contents = match fs::read_to_string(input_file) {
        Ok(c) => c,
        Err(e) => return Err(format!("{}\n{:?}", "Cannot read input file.", e)),
    };
    let output_file_contents = match replace_string(&rep_cont, input_file_contents, threshold) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    match fs::write(output_file, output_file_contents) {
        Ok(_) => (),
        Err(e) => return Err(format!("{}\n{:?}", "Cannot write to output file.", e)),
    };
    Ok(())
}

fn verify_rules(rep: &Replacements) -> Result<(), String> {
    for rule in rep.rules.iter() {
        if rule.honorifics.is_none() {
            // not a name replacement
            if rule.split == true {
                return Err(format!(
                    "{} [{}]: {}",
                    "Invalid Rule", rule.key, "Cannot split non-name replacements."
                ));
            }
        }
        match rep.contents.get(&rule.key) {
            Some(_) => (),
            None => {
                return Err(format!(
                    "Key [{}] from rules not found in replacements json",
                    rule.key
                ))
            }
        };
    }
    Ok(())
}

fn replace_string(
    rep: &Replacements,
    contents: String,
    threshold: usize,
) -> Result<String, String> {
    let mut output: String = contents;
    for rule in rep.rules.iter() {
        println!(
            "{} [{}]: {}",
            format!("* {}", rule.name).bold().green(),
            rule.key,
            rule.description.as_ref().unwrap_or(&String::from(""))
        );
        if rule.honorifics.is_none() {
            let rep_dict: HashMap<String, String> =
                match serde_json::from_str(&rep.contents.get(&rule.key).unwrap().to_string()) {
                    Ok(m) => m,
                    Err(e) => {
                        return Err(format!(
                            "{} [{}]: {}\n{:?}",
                            "Invalid JSON", &rule.key, "Cannot parse contents of key", e
                        ))
                    }
                };
            output = replace_hashmap(output, &rep_dict);
        } else {
            if rule.split {
                let (first_name, last_name) = rule.honorifics.unwrap();
                let rep_dict: HashMap<String, Vec<String>> =
                    match serde_json::from_str(&rep.contents.get(&rule.key).unwrap().to_string()) {
                        Ok(m) => m,
                        Err(e) => {
                            return Err(format!(
                                "{} [{}]: {}\n{:?}",
                                "Invalid JSON", &rule.key, "Cannot parse contents of key", e
                            ))
                        }
                    };
                output = match replace_names(
                    output,
                    &rep_dict,
                    &rep.honorifics,
                    first_name,
                    last_name,
                    threshold,
                ) {
                    Ok(o) => o,
                    Err(e) => return Err(e),
                };
            } else {
                let (first_name, _) = rule.honorifics.unwrap();
                let rep_dict: HashMap<String, String> =
                    match serde_json::from_str(&rep.contents.get(&rule.key).unwrap().to_string()) {
                        Ok(m) => m,
                        Err(e) => {
                            return Err(format!(
                                "{} [{}]: {}\n{:?}",
                                "Invalid JSON", &rule.key, "Cannot parse contents of key", e
                            ))
                        }
                    };
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
    return Ok(output);
}

fn replace_names(
    contents: String,
    rep_dict: &HashMap<String, Vec<String>>,
    honorifics: &HashMap<String, String>,
    first_name: bool,
    last_name: bool,
    threshold: usize,
) -> Result<String, String> {
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
        if en_names.len() != jp_names.len() {
            return Err(format!(
                "{}: {:?} {:?}",
                "English and Japanese names are different length for entry", en_names, jp_names
            ));
        }
        for i in 0..en_names.len() {
            for (hon_jp, hon_en) in honorifics.iter() {
                output = replace_single(
                    output,
                    &format!("{}{}", jp_names[i], hon_jp),
                    &format!("{}-{}", en_names[i], hon_en),
                );
            }
            if i == 0 && !first_name && jp_names[i].len() > threshold {
                output = replace_single(
                    output,
                    &format!("{}", jp_names[i]),
                    &format!("{}", en_names[i]),
                );
            } else if i == en_names.len() - 1 && !last_name && jp_names[i].len() > threshold {
                output = replace_single(
                    output,
                    &format!("{}", jp_names[i]),
                    &format!("{}", en_names[i]),
                );
            }
        }
    }
    return Ok(output);
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
        println!(
            "{} [{}] → {} ({})",
            find,
            find.len(),
            replace,
            format!("{}", count).bold()
        );
    }
    return output;
}
