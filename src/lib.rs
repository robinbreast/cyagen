//! # **C** code based **Y**et **A**nother **GEN**erator
//! - Scan C source file using the simple pattern matching to capture the elements in the code
//! - Generate text based files using template files
//! - Supported elements are inclusion, local variable, and functions
//!

extern crate chrono;
extern crate regex;

use chrono::Utc;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Include {
    pub captured: String,
}

#[derive(Debug)]
pub struct StaticVariable {
    pub captured: String,
    pub name_expr: String, // like "array[10]"
    pub name: String,      // like "array"
    pub dtype: String,
    pub is_local: bool,    // static variable declared within function
    pub func_name: String, // function name where local variable is declared
}

#[derive(Debug, Clone)]
pub struct Function {
    pub captured: String,
    pub name: String,
    pub is_local: bool,
    pub rtype: String,
    pub args: String,
    pub atypes: String,
}

#[derive(Debug)]
pub struct NestedCall {
    pub callee: Function,
    pub caller: Function,
}

#[derive(Debug)]
pub struct Parser {
    incs: Vec<Include>,
    static_vars: Vec<StaticVariable>,
    fncs: Vec<Function>,
    ncls: Vec<NestedCall>,
}

impl Parser {
    /// parse the given textdata and return Parse object to be used for generator
    ///
    pub fn parse<'a>(textdata: &'a str) -> Self {
        let code = remove_comments(textdata);
        let fncs = get_fncs(&code);
        let ncls = get_ncls(&code, &fncs);
        Self {
            incs: get_incs(&code),
            static_vars: get_static_vars(&code, &fncs),
            fncs: fncs.clone(),
            ncls: ncls,
        }
    }
}

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
        let re = Regex::new(r"@static-global-vars@(?P<fmt>[\S\s]*)@end-static-global-vars@").unwrap();
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

/// remove comments from C source code
///
fn remove_comments<'a>(code: &'a str) -> String {
    let re = Regex::new(r"(/\*([^*]|[\r\n]|(\*+([^*/]|[\r\n])))*\*+/)|(//.*)").unwrap();
    re.replace_all(&code, "").to_string()
}

/// list of inclusion from C source code
///
fn get_incs<'a>(code: &'a str) -> Vec<Include> {
    let mut result = vec![];
    let re = Regex::new(r#"(?P<captured>#include[\s]+["<].+[">])"#).unwrap();
    for cap in re.captures_iter(code) {
        result.push(Include {
            captured: cap.name("captured").unwrap().as_str().trim().to_string(),
        });
    }
    result.dedup();
    result
}

/// list of static variables from C source code
///
fn get_static_vars<'a>(code: &'a str, fncs: &Vec<Function>) -> Vec<StaticVariable> {
    let mut result = vec![];
    let re = Regex::new(r"(?i)(static)\s+(?P<captured>[^\(\{;=]+)[;=]").unwrap();
    for cap in re.captures_iter(code) {
        let captured = cap.name("captured").unwrap().as_str().trim().to_string();
        let name_expr = cap
            .name("captured")
            .unwrap()
            .as_str()
            .trim()
            .split(' ')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .trim_matches('*')
            .to_string();
        let name = if let Some(exprs) = name_expr.split_once('[') {
            exprs.0.to_string()
        } else {
            name_expr.clone()
        };
        let mut is_local = false;
        let mut func_name = String::from("");
        let data_type = cap
            .name("captured")
            .unwrap()
            .as_str()
            .trim()
            .strip_suffix(&name_expr)
            .unwrap()
            .trim()
            .to_string();
        for func in fncs {
            if let Some(pos) = code.find(func.captured.as_str()) {
                let start = pos + code.get(pos..).unwrap().find('{').unwrap() + 1;
                let stop = find_end_of_func(code, start);
                let body = code.get(start..stop).unwrap();
                if body.contains(captured.as_str()) {
                    is_local = true;
                    func_name = func.name.to_string();
                }
            }
        }
        result.push(StaticVariable {
            captured: captured,
            name_expr: name_expr,
            name: name,
            dtype: data_type,
            is_local: is_local,
            func_name: func_name,
        });
    }
    result
}

/// list of functions from C source code
///
fn get_fncs<'a>(code: &'a str) -> Vec<Function> {
    let mut result = vec![];
    let re = Regex::new(
        r"(?P<return>[[:alpha:]_][\w]*\s+)+(?P<name>[[:alpha:]_][\w]*)\s*\((?P<args>[^=!><>;\(\)-]*)\)\s*\{",
    ).unwrap();
    let get_atypes = |args: String| -> String {
        let mut type_list = String::new();
        let mut first_pos = true;
        let arg_list = args.split(',').collect::<Vec<&str>>();
        for arg in arg_list {
            let arg = arg.trim();
            if let Some(pos) = arg.rfind(|c: char| (c == '*') || (c == ' ')) {
                if first_pos {
                    first_pos = false;
                } else {
                    type_list.push_str(", ");
                }
                let mut tmpstr = arg.get(..(pos + 1)).unwrap().trim().to_string();
                let re4const = Regex::new(r"\w[\s\r\n]+const[\s\r\n]*\*").unwrap();
                if let Some(_) = re4const.captures(&tmpstr) {
                    let re4space = Regex::new(r"\s+").unwrap();
                    let pos = tmpstr.find("const").unwrap();
                    tmpstr.replace_range(pos..(pos + 5), "");
                    tmpstr = format!("const {}", tmpstr);
                    tmpstr = re4space.replace_all(&tmpstr, " ").to_string();
                }
                type_list.push_str(&tmpstr);
                let array_dimension = arg.get(pos..).unwrap().matches("[").count();
                type_list.push_str(&"*".repeat(array_dimension));
            }
        }
        if type_list.trim() == "void" {
            type_list.clear();
        }
        type_list
    };
    for cap in re.captures_iter(code) {
        if cap.name("name").unwrap().as_str().trim() == "if" {
            continue;
        }
        let re4space = Regex::new(r"\s+").unwrap();
        let mut raw_args = re4space
            .replace_all(cap.name("args").unwrap().as_str().trim(), " ")
            .replace("\\", "")
            .trim()
            .to_string();
        if raw_args.trim() == "void" {
            raw_args.clear();
        }
        result.push(Function {
            captured: cap.get(0).unwrap().as_str().trim().to_string(),
            name: cap.name("name").unwrap().as_str().trim().to_string(),
            is_local: cap
                .get(0)
                .unwrap()
                .as_str()
                .to_ascii_lowercase()
                .contains("static"),
            rtype: cap
                .name("return")
                .unwrap()
                .as_str()
                .replace("static", "")
                .replace("STATIC", "")
                .replace("inline", "")
                .replace("INLINE", "")
                .trim()
                .to_string(),
            args: raw_args.clone(),
            atypes: get_atypes(raw_args.clone()),
        });
    }
    result
}

/// find end of func
///
fn find_end_of_func<'a>(code: &'a str, start: usize) -> usize {
    let mut stop = start;
    let mut open = 1;
    for (i, c) in code.get(start..).unwrap().chars().enumerate() {
        if c == '}' {
            open -= 1;
        } else if c == '{' {
            open += 1;
        }
        if open == 0 {
            stop += i;
            break;
        }
    }
    stop
}

/// list of ncls in C source
///
fn get_ncls<'a>(code: &'a str, fncs: &Vec<Function>) -> Vec<NestedCall> {
    let mut result = vec![];
    for caller in fncs {
        if let Some(pos) = code.find(caller.captured.as_str()) {
            let start = pos + code.get(pos..).unwrap().find('{').unwrap() + 1;
            let stop = find_end_of_func(code, start);
            let body = code.get(start..stop).unwrap();
            for callee in fncs {
                let call_str = format!("{}(", callee.name);
                if body.contains(call_str.as_str()) {
                    result.push(NestedCall {
                        callee: callee.clone(),
                        caller: caller.clone(),
                    });
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    static TEST_CODE: &'static str = "\
#include <stdio.h>
#include <stdio.h>
#include \"test.h\"

int global_var = 2;
static char static_var;

// test-comment1
/* test-comment2 */
/* 
    test-comment3
*/

static inline char local_function(int a);

void main()
{
    char c = local_function(20);
    if (c == 1)
    {
        printf(\"no operation\");
    }
    else
    {
        printf(\"hello world! %c\n\", c);
    }
}

static inline char local_function(int a, 
    int*b )
{
    static int local_var[10];
    return (char)a;
}
";

    #[test]
    fn test_parse() {
        let code = fs::read_to_string("sample.c").unwrap();
        let parser = Parser::parse(&code);
        println!("{:#?}", parser);
    }

    #[test]
    fn test_remove_comments() {
        let clean_code = remove_comments(TEST_CODE);
        assert!(!clean_code.contains("test-comment"));
    }

    #[test]
    fn test_get_incs() {
        let list_incs = get_incs(TEST_CODE);
        assert_eq!(list_incs[0].captured, "#include <stdio.h>");
        assert_eq!(list_incs[1].captured, "#include \"test.h\"");
    }

    #[test]
    fn test_get_static_vars() {
        let list_fncs = get_fncs(TEST_CODE);
        let list_static_vars = get_static_vars(TEST_CODE, &list_fncs);
        assert_eq!(list_static_vars[0].name, "static_var");
        assert_eq!(list_static_vars[0].dtype, "char");
        assert_eq!(list_static_vars[0].is_local, false);
        assert_eq!(list_static_vars[1].name, "local_var");
        assert_eq!(list_static_vars[1].name_expr, "local_var[10]");
        assert_eq!(list_static_vars[1].dtype, "int");
        assert_eq!(list_static_vars[1].is_local, true);
        assert_eq!(list_static_vars[1].func_name, "local_function");
    }

    #[test]
    fn test_get_fncs() {
        let list_fncs = get_fncs(TEST_CODE);
        assert_eq!(list_fncs[0].name, "main");
        assert_eq!(list_fncs[0].rtype, "void");
        assert!(!list_fncs[0].is_local);
        assert_eq!(list_fncs[1].name, "local_function");
        assert_eq!(list_fncs[1].rtype, "char");
        assert_eq!(list_fncs[1].atypes, "int, int*");
        assert!(list_fncs[1].is_local);
    }

    #[test]
    fn test_get_ncls() {
        let list_fncs = get_fncs(TEST_CODE);
        let list_ncls = get_ncls(TEST_CODE, &list_fncs);
        if list_ncls.len() > 0 {
            assert_eq!(list_ncls[0].caller.name, "main");
            assert_eq!(list_ncls[0].callee.name, "local_function");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_generate_incs() {
        let sourcename = "test";
        let code = "\
#include <header1.h>
#include <header2.h>
";
        let temp = "\
// include
@incs@@captured@
@end-incs@
";
        let expected = "\
// include
#include <header1.h>
#include <header2.h>

";
        let parser = Parser::parse(code);
        let generated = generate(&parser, temp, sourcename);
        assert_eq!(generated, expected);
    }

    #[test]
    fn test_generate_static_vars() {
        let sourcename = "test";
        let code = "\
static int a[10];
int b;
static char *c;
void func1(void)
{
    static int local_var;
}
";
        let temp = "\
// static variables
@static-vars@@dtype@ @name-expr@;
@end-static-vars@
// static global variables
@static-global-vars@@dtype@ @name@;
@end-static-global-vars@
// static local variables
@static-local-vars@@dtype@ @name@;
@end-static-local-vars@
";
        let expected = "\
// static variables
int a[10];
char * c;
int local_var;

// static global variables
int a;
char * c;

// static local variables
int local_var;

";
        let parser = Parser::parse(code);
        let generated = generate(&parser, temp, sourcename);
        assert_eq!(generated, expected);
    }

    #[test]
    fn test_generate_fncs() {
        let sourcename = "test";
        let code = "\
// functions
int func1()
{
    return 0;
}
void func2(int const * a)
{
}
";
        let temp = "\
// functions
@fncs@@rtype@ @name@(@args@);
@atypes@
@end-fncs@
";
        let expected = "\
// functions
int func1();

void func2(int const * a);
const int *

";
        let parser = Parser::parse(code);
        let generated = generate(&parser, temp, sourcename);
        assert_eq!(generated, expected);
    }

    #[test]
    fn test_generate_ncls() {
        let sourcename = "test";
        let code = "\
// functions
void func1()
{
    return;
}
int func2(int a)
{
    return func1();
}
";
        let temp = "\
@ncls@- @caller.name@ -> @callee.name@
    - return @callee.rtype.remove(0)@;
    - (int dummy@callee.args.remove(, )@@callee.args@);
@end-ncls@
";
        let expected = "\
- func2 -> func1
    - return ;
    - (int dummy);

";
        let parser = Parser::parse(code);
        let generated = generate(&parser, temp, sourcename);
        assert_eq!(generated, expected);
    }

    #[test]
    fn test_generate_ncls_once() {
        let sourcename = "test";
        let code = "\
// functions
int func1()
{
    return 0;
}
void func2(int a)
{
    func1();
}
void func3(int a)
{
    func1();
}
";
        let temp = "\
@ncls-once@- @callee.name@
@end-ncls-once@
";
        let expected = "\
- func1

";
        let parser = Parser::parse(code);
        let generated = generate(&parser, temp, sourcename);
        assert_eq!(generated, expected);
    }
}
