# cyagen

## **C** code based **Y**et **A**nother **GEN**erator

File generator to reduce the manual effort to prepare another scripting files which contain C code information derived by a C source file.

- Scan C source file using the simple pattern matching to capture the elements in the code
- Generate text based files using template files
- Supported elements are inclusion, local variable, and functions

## Better use jinja2 format in template files
- Since 0.1.19, cyagen supports jinja2 format of template files using [tera](https://crates.io/crates/tera)
- if a template file extension is .j2 or .tera, cyagen generates target files using **tera** engine
- example: **to create googletest script skeleton and CMakeLists.txt based on C code**
```
$ cyagen --source ./example/source/sample.c --temp-dir ./example/template --output-dir ./.output
$ cd .output
$ cmake -S . -B build
$ cmake --build build
$ cd build && ctest
```

## Available identifiers in a template file
All the available identifiers can be found on [docs.rs](https://docs.rs/crate/cyagen)
> Notice: all the new identifiers are not supported on the old style of template (not jinja2 format).
 
## Example

```
let sourcename = "source";
let code = "\
#include <stdio.h>
static int var = 1;
static int func1(void)
{
    return 0;
}
int func2(char c)
{
    return func1();
}
";
let temp = "\
// include
{%- for inc in incs %}
{{ inc.captured | safe }}
{%- endfor %}

// local variables
{%- for var in static_vars %}
{{ var.dtype }} {{ var.name }};
{%- endfor %}

// functions
{%- for fnc in fncs %}
{{ fnc.rtype }} {{ fnc.name }}({{ fnc.args }});
{%- endfor %}
";
let parser = cyagen::Parser::parse(code);
let gen = cyagen::generate_using_tera(&parser, temp, sourcename);
println!("{}", gen);
```
## Result

```
// include
#include <stdio.h>

// local variables
int var;

// functions
int func1();
int func2(char c);

```

## Default application command line usage
```
$ cyagen --help
cyagen 0.1.10
Text file generator based on C file and templates

USAGE:
    cyagen [OPTIONS] --source <SOURCE>

OPTIONS:
    -s, --source <SOURCE>               source file path
    -t, --temp-dir <TEMP_DIR>           template directory
    -o, --output-dir <OUTPUT_DIR>       output directory
    -j, --json-filepath <JSON_FILEPATH> output json file path
    -h, --help                          Print help
    -V, --version                       Print version
$
```