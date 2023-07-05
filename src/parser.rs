use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;

/// identifiers in the list of "include"
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Include {
    /// captured string by regex; ex) "#include <stdio.h>"
    pub captured: String,
}

/// identifiers in the list of "typedef"
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Typedefs {
    /// captured string by regex; ex) "typedef enum\n{\n    Idle = 0,\n    Forward,\n    TurnLeft,\n    TurnRight,\n    MaxDirection\n} Direction_t;"
    pub captured: String,
}

/// identifiers in the list of "static_vars"
#[derive(Debug, Serialize, Deserialize)]
pub struct StaticVariable {
    /// captured string by regex; ex) "static int array_var\[10\] = { 1, 2, 3 };"
    pub captured: String,
    /// variable name expression; ex) "array_var\[10\]"
    pub name_expr: String,
    /// variable name; ex) "array_var"
    pub name: String,
    /// data type; ex) "int"
    pub dtype: String,
    /// true if the variable is declared within a function
    pub is_local: bool,
    /// function where the variable is declared
    pub func_name: String,
    /// init value; ex) "{ 1, 2, 3 }"
    pub init: String,
    /// array size; ex) "10"
    pub array_size: i32,
    /// true if the variable is const
    pub is_const: bool,
}

/// identifiers in the list of "fncs"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// captured string by regex; ex) "void setDir(const Direction_t dir)\n{"
    pub captured: String,
    /// name; ex) "setDir"
    pub name: String,
    /// true if the function is static; ex) false
    pub is_local: bool,
    /// return data type; ex) "void"
    pub rtype: String,
    /// arguments; ex) "const Direction_t dir"
    pub args: String,
    /// argument data types; ex) "const Direction_t"
    pub atypes: String,
    /// argument names; ex) "dir"
    pub anames: String,
}

/// identifiers in the list of "ncls"
#[derive(Debug, Serialize, Deserialize)]
pub struct NestedCall {
    pub callee: Function,
    pub caller: Function,
}

/// identifiers which can be used in template
#[derive(Debug, Serialize, Deserialize)]
pub struct Parser {
    pub json_object: serde_json::Value,
    /// source filename without extension to be parsed
    pub sourcename: String,
    /// source file folder
    pub sourcedirname: String,
    /// local static variable macro name; LOCAL_STATIC_VARIABLE
    pub lsv_macro_name: String,
    /// list of "include"
    pub incs: Vec<Include>,
    /// list of "typedef"
    pub typedefs: Vec<Typedefs>,
    /// list of static variables
    pub static_vars: Vec<StaticVariable>,
    /// list of functions
    pub fncs: Vec<Function>,
    /// list of nested calls
    pub ncls: Vec<NestedCall>,
    /// list of functions called within the source file
    pub callees: Vec<Function>,
}

impl Parser {
    /// parse the given textdata and return Parse object to be used for generator
    ///
    pub fn parse(textdata: &str) -> Self {
        let code = remove_comments(textdata);
        let fncs = get_fncs(&code);
        let ncls = get_ncls(&code, &fncs);
        let callees: Vec<Function> = get_callees(&ncls);
        let mut static_vars = get_static_vars(&code, &fncs);
        let lsv_macro_name = "LOCAL_STATIC_VARIABLE".to_string();
        update_static_vars_with_lsv(&code, &fncs, &lsv_macro_name, &mut static_vars);
        Self {
            json_object: serde_json::json!({}),
            sourcename: String::new(),
            sourcedirname: String::new(),
            lsv_macro_name: lsv_macro_name,
            incs: get_incs(&code),
            typedefs: get_typedefs(&code),
            static_vars: static_vars,
            fncs: fncs.clone(),
            ncls: ncls,
            callees: callees,
        }
    }
}

/// remove comments from C source code
///
fn remove_comments(code: &str) -> String {
    let re = Regex::new(r"(/\*([^*]|[\r\n]|(\*+([^*/]|[\r\n])))*\*+/)|(//.*)").unwrap();
    re.replace_all(&code, "").to_string()
}

/// list of inclusion from C source code
///
fn get_incs(code: &str) -> Vec<Include> {
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

/// list of typedef from C source code
///
fn get_typedefs(code: &str) -> Vec<Typedefs> {
    let mut result = vec![];
    let re = Regex::new(r#"(?P<captured>typedef\s+(?:.*?\{[.\s\S]*?\}.*?;|[.\s\S]+?;))"#).unwrap();
    for cap in re.captures_iter(code) {
        result.push(Typedefs {
            captured: cap.name("captured").unwrap().as_str().trim().to_string(),
        });
    }
    result.dedup();
    result
}

/// update the list of static variables with LOCAL_STATIC_VARIABLE string pattern
///
fn update_static_vars_with_lsv(
    code: &str,
    fncs: &Vec<Function>,
    lsv_macro_name: &str,
    static_vars: &mut Vec<StaticVariable>,
) {
    let regex_str = format!(
        "{}\\((?<fnc_name>\\w+)\\s*,(?<dtype>.*?)\\s*,\\s*(?<name>\\w+)\\s*(?:\\[(?<array_size>.*?)\\])?\\s*,\\s*(?<value>.*?)\\).*?;",
        &lsv_macro_name
    );
    let re = Regex::new(&regex_str).unwrap();
    for cap in re.captures_iter(code) {
        let captured = cap.get(0).unwrap().as_str().trim().to_string();
        let dtype = cap.name("dtype").unwrap().as_str().trim().to_string();
        let name = cap.name("name").unwrap().as_str().trim().to_string();
        let array_size = cap
            .name("array_size")
            .map_or(0, |c| c.as_str().parse().unwrap_or(0));
        let init = cap
            .name("value")
            .map_or("0", |c| c.as_str().trim())
            .to_string();
        let is_const = cap
            .name("dtype")
            .map_or(false, |c| c.as_str().to_lowercase().contains("const"));
        let name_expr = cap.name("array_size").map_or(name.clone(), |c| {
            (name.clone() + "[" + c.as_str().trim() + "]").to_string()
        });
        let mut is_local = false;
        let mut func_name = String::from("");
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
        static_vars.push(StaticVariable {
            captured: captured,
            name_expr: name_expr,
            name: name,
            dtype: dtype,
            is_local: is_local,
            func_name: func_name,
            init: init,
            array_size: array_size,
            is_const: is_const,
        });
    }
}

/// list of static variables from C source code
///
fn get_static_vars(code: &str, fncs: &Vec<Function>) -> Vec<StaticVariable> {
    let mut result = vec![];
    let re = Regex::new(r"(?i)(?<keyword>static\s+|static\s+const\s+|const\s+static\s+)+(?<dtype>.*?)(?<name>\w+)\s*(?:\[(?<array_size>.*?)\])?\s*(?:=\s*(?<value>\{.*?\}|.*?))?;").unwrap();
    for cap in re.captures_iter(code) {
        let captured = cap.get(0).unwrap().as_str().trim().to_string();
        let dtype = cap.name("dtype").unwrap().as_str().trim().to_string();
        let name = cap.name("name").unwrap().as_str().trim().to_string();
        let array_size = cap
            .name("array_size")
            .map_or(0, |c| c.as_str().parse().unwrap_or(0));
        let init = cap
            .name("value")
            .map_or("0", |c| c.as_str().trim())
            .to_string();
        let is_const = cap
            .name("keyword")
            .map_or(false, |c| c.as_str().to_lowercase().contains("const"));
        let name_expr = cap.name("array_size").map_or(name.clone(), |c| {
            (name.clone() + "[" + c.as_str().trim() + "]").to_string()
        });
        let mut is_local = false;
        let mut func_name = String::from("");
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
            dtype: dtype,
            is_local: is_local,
            func_name: func_name,
            init: init,
            array_size: array_size,
            is_const: is_const,
        });
    }
    result
}

/// list of functions from C source code
///
fn get_fncs(code: &str) -> Vec<Function> {
    let mut result = vec![];
    let re = Regex::new(
        r"((?<return>\w+[\w\s\*]*\s+)|FUNC\((?<return_ex>[^,]+),[^\)]+\)\s*)(?<name>\w+)[\w]*\s*\((?<args>[^=!><>;\(\)-]*)\)\s*\{"
    ).unwrap();
    let get_atypes = |args: String| -> (String, String) {
        let mut type_list = String::new();
        let mut name_list = String::new();
        let mut first_pos = true;
        let arg_list = args.split(',').collect::<Vec<&str>>();
        for arg in arg_list {
            let arg = arg.trim();
            let re4sep = Regex::new(r"^(?<atype>.*?)(?<aname>\w+(?:\[.*?\])*)$").unwrap();
            let mut atype;
            let mut aname;
            if let Some(cap) = re4sep.captures(&arg) {
                atype = cap.name("atype").unwrap().as_str().trim().to_string();
                aname = cap.name("aname").unwrap().as_str().trim().to_string();
                // relocate 'const' only for 'datatype const *' -> 'const datatype *'
                let re4const = Regex::new(r"\w[\s\r\n]+const[\s\r\n]*\*").unwrap();
                if let Some(_) = re4const.captures(&atype) {
                    atype = atype.replace("const", "");
                    atype = format!("const {}", atype);
                    let re4space = Regex::new(r"\s+").unwrap();
                    atype = re4space.replace_all(&atype, " ").to_string();
                }
                if first_pos {
                    first_pos = false;
                } else {
                    type_list.push_str(", ");
                    name_list.push_str(", ");
                }
                type_list.push_str(&atype);
                let array_dimension = aname.matches("[").count();
                type_list.push_str(&"*".repeat(array_dimension));
                // remove '[]' from name string
                let re4bracket = Regex::new(r"(\[.*?\])+").unwrap();
                if let Some(_) = re4bracket.captures(&aname) {
                    aname = re4bracket.replace_all(&aname, "").to_string();
                }
                name_list.push_str(&aname);
            }
        }
        if type_list.trim() == "void" {
            type_list.clear();
            name_list.clear();
        }
        (type_list, name_list)
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
        let (atypes, anames) = get_atypes(raw_args.clone());
        let rtype = cap
            .name("return")
            .or(cap.name("return_ex"))
            .unwrap()
            .as_str()
            .replace("static", "")
            .replace("STATIC", "")
            .replace("inline", "")
            .replace("INLINE", "")
            .trim()
            .to_string();
        result.push(Function {
            captured: cap.get(0).unwrap().as_str().trim().to_string(),
            name: cap.name("name").unwrap().as_str().trim().to_string(),
            is_local: cap
                .get(0)
                .unwrap()
                .as_str()
                .to_ascii_lowercase()
                .contains("static"),
            rtype: rtype,
            args: raw_args.clone(),
            atypes: atypes,
            anames: anames,
        });
    }
    result
}

/// find end of func
///
fn find_end_of_func(code: &str, start: usize) -> usize {
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
fn get_ncls(code: &str, fncs: &Vec<Function>) -> Vec<NestedCall> {
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

fn find_func_in_list(funcname: &str, fncs: &Vec<Function>) -> bool {
    let mut result = false;
    for fnc in fncs {
        if funcname == fnc.name {
            result = true;
            break;
        }
    }
    result
}

fn get_callees(ncls: &Vec<NestedCall>) -> Vec<Function> {
    let mut result: Vec<Function> = vec![];
    for ncl in ncls {
        if !find_func_in_list(&ncl.callee.name, &result) {
            result.push(ncl.callee.clone());
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
        let code = fs::read_to_string("./example/source/sample.c").unwrap();
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
}
