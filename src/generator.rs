use super::parser::Parser;

use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use serde_json;
use std::fs;

use tera;
use uuid::Uuid;

const NAMESPACE_OID: Uuid = Uuid::from_u128(0x6ba7b812_9dad_11d1_80b4_00c04fd430c8);

fn generate_uuid(
    value: &tera::Value,
    _: &std::collections::HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let uuid: Uuid = match value {
        tera::Value::String(s) => Uuid::new_v5(&NAMESPACE_OID, s.as_bytes()),
        _ => return Err(tera::Error::msg("Invalid value")),
    };
    Ok(tera::to_value(uuid.hyphenated().to_string()).unwrap())
}

/// Function to generate UUID
pub fn generate_using_tera<'a>(parser: &'a Parser, template: &'a str) -> String {
    let mut tera = tera::Tera::default();

    // register filter function
    tera.register_filter("generateUUID", generate_uuid);

    // prepare context from parser
    let json_data: tera::Value = serde_json::to_value(&parser).unwrap();
    let mut context = tera::Context::new();
    for (key, value) in json_data.as_object().unwrap() {
        context.insert(key, value);
    }

    // render template
    let result = tera.render_str(&template, &context);
    match result {
        Ok(value) => value,
        Err(error) => panic!("Error: {}", error),
    }
}

pub fn generate_json<'a>(parser: &'a Parser, filepath: &'a String) -> Result<()> {
    let file = fs::File::create(filepath)
        .with_context(|| format!("failed to create file `{}`", filepath))?;
    serde_json::to_writer(file, parser)?;
    Ok(())
}

pub fn merge_with_manual_sections(rendered: &str, old_gen: &str) -> String {
    let regex = Regex::new(r"(?s)MANUAL SECTION: ([a-f0-9-]+).*?MANUAL SECTION END").unwrap();
    let merged = regex.replace_all(&rendered, |captures: &regex::Captures<'_>| {
        let uuid = &captures[1];
        let manual_content = Regex::new(&format!(
            "(?s)MANUAL SECTION: {}.*?MANUAL SECTION END",
            uuid
        ))
        .unwrap();
        manual_content
            .find(old_gen)
            .map_or(captures[0].to_string(), |m| m.as_str().to_string())
    });

    merged.into_owned()
}

/// DUE-TO-BACKWARD-COMPATIBILITY
/// generate document based on parsing result and template data
///
/// # Available tags in template file
/// - **@sourcename@** : it is given as an argument from command line
/// - **@date@** : generated date
/// - **@incs@** : the list of inclusion statement such as `#include <stdio.h>`
///     - **@captured@** : the captured raw string
/// - **@end-incs@** : the end of **incs** block
/// - **@static-vars@** or **@static-global-vars@** or **@static-local-vars@**: the list of **static** variables
///     - **@captured@** : the captured raw string
///     - **@name@** : variable name
///     - **@name-expr@** : variable name including brackets when array data
///     - **@dtype@** : variable data type
///     - **@func-name@** : function name only for **static-local-vars**
/// - **@end-static-vars@** or **@end-static-global-vars@** or **@end-static-local-vars@**: the end of **static-vars** bolck
/// - **@fncs@** or **@fncs0@** : the list of all the functions
///     - **@captured@** : the captured raw string
///     - **@name@** : the function name
///     - **@rtype@** : the return data type of the function
///     - **@args@** : the list of arguments with data types
///     - **@atypes@** : the list of only arguments' data types
/// - **@end-fncs@** or **@end-fncs0@** : the end of **fncs** or **fncs0** block
/// - **@ncls@** or **@ncls-once@** : the list of nested calls, no duplicate callee with **ncls-once**
///     - **@callee.name@** : the function name of callee
///     - **@callee.rtype@** : the return type of callee
///     - **@callee.rtype.change(\<from\>=\<to\>)@** : to change return data type during generation
///     - **@callee.rtype.remove(\<text\>)@** : \<text\> to be removed when `void`
///     - **@callee.rtype.remove0(\<text\>)@** : \<text\> to be removed when `void`
///     - **@callee.args@** : the argument list string
///     - **@callee.args.remove(\<text\>)@** : \<text\> to be removed when `void`
///     - **@callee.atypes@** : only arguments' data types
///     - **@caller.name@** : the function name of caller
///     - **@caller.rtype@** : the return type of caller
///     - **@caller.args@** : the argument list string
///     - **@caller.atypes@** : only arguments' data types
/// - **@end-ncls@** or **@end-ncls-once@** : the end of **ncls** or **ncls-once** block
///
/// # Example
///
/// ```
/// let sourcename = "source";
/// let code = "\
/// #include <stdio.h>
/// static int var = 1;
/// static int func1(void)
/// {
///     return 0;
/// }
/// int func2(char c)
/// {
///     return func1();
/// }
/// ";
/// let temp = "\
/// // include
/// @incs@@captured@
/// @end-incs@
/// // local variables
/// @local-vars@@dtype@ @name@;
/// @end-local-vars@
/// // functions
/// @fncs@@rtype@ @name@(@args@);
/// @end-fncs@
/// ";
/// let parser = cyagen::Parser::parse(code);
/// let gen = cyagen::generate(&parser, temp, sourcename);
/// ```
pub fn generate<'a>(parser: &'a Parser, template: &'a str, sourcename: &'a str) -> String {
    let mut output = String::from(template);
    if template.contains("@incs@") {
        let re = Regex::new(r"@incs@(?P<fmt>[\S\s]*)@end-incs@").unwrap();
        for cap in re.captures_iter(template) {
            let mut tmpstr = String::new();
            for entry in &parser.incs {
                let fmtstr = cap
                    .name("fmt")
                    .unwrap()
                    .as_str()
                    .replace("@captured@", &entry.captured);
                tmpstr.push_str(&fmtstr);
            }
            output = re.replace(&output, tmpstr.as_str()).into_owned();
        }
    }
    if template.contains("@static-vars@") {
        let re = Regex::new(r"@static-vars@(?P<fmt>[\S\s]*)@end-static-vars@").unwrap();
        for cap in re.captures_iter(template) {
            let mut tmpstr = String::new();
            for entry in &parser.static_vars {
                let fmtstr = cap
                    .name("fmt")
                    .unwrap()
                    .as_str()
                    .replace("@captured@", &entry.captured)
                    .replace("@name@", &entry.name)
                    .replace("@name-expr@", &entry.name_expr)
                    .replace("@dtype@", &entry.dtype);
                tmpstr.push_str(&fmtstr);
            }
            output = re.replace(&output, tmpstr.as_str()).into_owned();
        }
    }
    if template.contains("@static-global-vars@") {
        let re =
            Regex::new(r"@static-global-vars@(?P<fmt>[\S\s]*)@end-static-global-vars@").unwrap();
        for cap in re.captures_iter(template) {
            let mut tmpstr = String::new();
            for entry in &parser.static_vars {
                if !entry.is_local {
                    let fmtstr = cap
                        .name("fmt")
                        .unwrap()
                        .as_str()
                        .replace("@captured@", &entry.captured)
                        .replace("@name@", &entry.name)
                        .replace("@name-expr@", &entry.name_expr)
                        .replace("@dtype@", &entry.dtype);
                    tmpstr.push_str(&fmtstr);
                }
            }
            output = re.replace(&output, tmpstr.as_str()).into_owned();
        }
    }
    if template.contains("@static-local-vars@") {
        let re = Regex::new(r"@static-local-vars@(?P<fmt>[\S\s]*)@end-static-local-vars@").unwrap();
        for cap in re.captures_iter(template) {
            let mut tmpstr = String::new();
            for entry in &parser.static_vars {
                if entry.is_local {
                    let fmtstr = cap
                        .name("fmt")
                        .unwrap()
                        .as_str()
                        .replace("@captured@", &entry.captured)
                        .replace("@name@", &entry.name)
                        .replace("@name-expr@", &entry.name_expr)
                        .replace("@func-name@", &entry.func_name)
                        .replace("@dtype@", &entry.dtype);
                    tmpstr.push_str(&fmtstr);
                }
            }
            output = re.replace(&output, tmpstr.as_str()).into_owned();
        }
    }
    let fncs_tags = vec!["fncs", "fncs0"];
    for tag in fncs_tags {
        let regstr = format!("@{}@{}@end-{}@", tag, r"(?P<fmt>[\S\s]*)", tag);
        let re = Regex::new(&regstr).unwrap();
        for cap in re.captures_iter(template) {
            let mut tmpstr = String::new();
            for entry in &parser.fncs {
                let fmtstr = cap
                    .name("fmt")
                    .unwrap()
                    .as_str()
                    .replace("@captured@", &entry.captured)
                    .replace("@name@", &entry.name)
                    .replace("@rtype@", &entry.rtype)
                    .replace("@args@", &entry.args)
                    .replace("@atypes@", &entry.atypes);
                tmpstr.push_str(&fmtstr);
            }
            output = re.replace(&output, tmpstr.as_str()).into_owned();
        }
    }
    if output.contains("@local-fncs@") {
        let re = Regex::new(r"@local-fncs@(?P<fmt>[\S\s]*)@end-local-fncs@").unwrap();
        for cap in re.captures_iter(template) {
            let mut tmpstr = String::new();
            for entry in &parser.fncs {
                if entry.is_local {
                    let fmtstr = cap
                        .name("fmt")
                        .unwrap()
                        .as_str()
                        .replace("@captured@", &entry.captured)
                        .replace("@name@", &entry.name)
                        .replace("@rtype@", &entry.rtype)
                        .replace("@args@", &entry.args)
                        .replace("@atypes@", &entry.atypes);
                    tmpstr.push_str(&fmtstr);
                }
            }
            output = re.replace(&output, tmpstr.as_str()).into_owned();
        }
    }
    let ncls_tags = vec!["ncls", "ncls-once"];
    for tag in ncls_tags {
        let regstr = format!("@{}@{}@end-{}@", tag, r"(?P<fmt>[\S\s]*)", tag);
        let re = Regex::new(&regstr).unwrap();
        for cap in re.captures_iter(template) {
            let mut tmpstr = String::new();
            let mut callee_list: Vec<String> = Vec::new();
            for entry in &parser.ncls {
                if tag == "ncls-once" {
                    if callee_list.contains(&entry.callee.name) {
                        continue;
                    }
                    callee_list.push(entry.callee.name.to_string());
                }
                let mut fmtstr = cap
                    .name("fmt")
                    .unwrap()
                    .as_str()
                    .replace("@callee.name@", &entry.callee.name)
                    .replace("@callee.rtype@", &entry.callee.rtype)
                    .replace("@callee.args@", &entry.callee.args)
                    .replace("@callee.atypes@", &entry.callee.atypes)
                    .replace("@caller.name@", &entry.caller.name)
                    .replace("@caller.rtype@", &entry.caller.rtype)
                    .replace("@caller.args@", &entry.caller.args)
                    .replace("@caller.atypes@", &entry.caller.atypes);
                let re4change = Regex::new(
                    r"@callee.rtype.change\((?P<from>[a-z|A-Z|0-9|_]+)=(?P<to>[a-z|A-Z|0-9|_]+)\)@",
                )
                .unwrap();
                for cap in re4change.captures_iter(fmtstr.clone().as_str()) {
                    if cap.name("from").unwrap().as_str() == entry.callee.rtype.as_str() {
                        let to = cap.name("to").unwrap().as_str();
                        fmtstr = re4change.replace(&fmtstr, to).into_owned();
                    } else {
                        fmtstr = re4change.replace(&fmtstr, &entry.callee.rtype).into_owned();
                    }
                }
                // remove tags for callee.rtype
                let remove_tags = vec!["callee.rtype.remove", "callee.rtype.remove0"];
                for tag in remove_tags {
                    let regstr = format!(r"@{}\((?P<text>[^)]+)\)@", tag);
                    let re = Regex::new(&regstr).unwrap();
                    for cap in re.captures_iter(fmtstr.clone().as_str()) {
                        if entry.callee.rtype.as_str() == "void" {
                            fmtstr = re.replace(&fmtstr, "").into_owned();
                        } else {
                            let text = cap.name("text").unwrap().as_str();
                            fmtstr = re.replace(&fmtstr, text).into_owned();
                        }
                    }
                }
                // remove tag for callee.args
                let remove_tag = "callee.args.remove";
                let regstr = format!(r"@{}\((?P<text>[^)]+)\)@", remove_tag);
                let re = Regex::new(&regstr).unwrap();
                for cap in re.captures_iter(fmtstr.clone().as_str()) {
                    if entry.callee.args.as_str() == "void" || entry.callee.args.as_str() == "" {
                        fmtstr = re.replace(&fmtstr, "").into_owned();
                    } else {
                        let text = cap.name("text").unwrap().as_str();
                        fmtstr = re.replace(&fmtstr, text).into_owned();
                    }
                }
                tmpstr.push_str(&fmtstr);
            }
            output = re.replace(&output, tmpstr.as_str()).into_owned();
        }
    }
    output.replace("@sourcename@", sourcename).replace(
        "@date@",
        Utc::now().format("%a %b %e %T %Y").to_string().as_str(),
    )
}
