# cyagen

## **C** code based **Y**et **A**nother **GEN**erator

File generator to reduce the manual effort to prepare another scripting files which contain C code information derived by a C source file.

- Scan C source file using the simple pattern matching to capture the elements in the code
- Generate text based files using template files
- Supported elements are inclusion, local variable, and functions

## Available tags in a template file
- **@sourcename@** : it is given as an argument from command line
- **@date@** : generated date
- **@incs@** : the list of inclusion statement such as `#include <stdio.h>`
    - **@captured@** : the captured raw string
- **@end-incs@** : the end of **incs** block
- **@static-vars@** or **@static-global-vars@** or **@static-local-vars@**: the list of **static** variables
    - **@captured@** : the captured raw string
    - **@name@** : variable name
    - **@name-expr@** : variable name including brackets when array data
    - **@dtype@** : variable data type
    - **@func-name@** : function name only for **static-local-vars**
- **@end-static-vars@** or **@end-static-global-vars@** or **@end-static-local-vars@**: the end of **static-vars** bolck
- **@fncs@** or **@fncs0@** : the list of all the functions
    - **@captured@** : the captured raw string
    - **@name@** : the function name
    - **@rtype@** : the return data type of the function
    - **@args@** : the list of arguments with data types
    - **@atypes@** : the list of only arguments' data types
- **@end-fncs@** or **@end-fncs0@** : the end of **fncs** or **fncs0** block
- **@ncls@** or **@ncls-once@** : the list of nested calls, no duplicate callee with **ncls-once**
    - **@callee.name@** : the function name of callee
    - **@callee.rtype@** : the return type of callee
    - **@callee.rtype.change(\<from\>=\<to\>)@** : to change return data type during generation
    - **@callee.rtype.remove(\<text\>)@** : \<text\> to be removed when `void`
    - **@callee.rtype.remove0(\<text\>)@** : \<text\> to be removed when `void`
    - **@callee.args@** : the argument list string
    - **@callee.args.remove(\<text\>)@** : \<text\> to be removed when `void`
    - **@callee.atypes@** : only arguments' data types
    - **@caller.name@** : the function name of caller
    - **@caller.rtype@** : the return type of caller
    - **@caller.args@** : the argument list string
    - **@caller.atypes@** : only arguments' data types
- **@end-ncls@** or **@end-ncls-once@** : the end of **ncls** or **ncls-once** block
 
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
@incs@@captured@
@end-incs@
// local variables
@local-vars@@dtype@ @name@;
@end-local-vars@
// functions
@fncs@@rtype@ @name@(@args@);
@end-fncs@
";
let parser = cyagen::Parser::parse(code);
let gen = cyagen::generate(&parser, temp, sourcename);
println!("{}", gen);
```
## Result

```
// include
#include <stdio.h>

// local variables
int var;

// functions
int func1(void);
int func2(char c);

```

## Default application command line usage
```
$ cyagen --help
cyagen 0.1.8
Text file generator based on C file and templates

USAGE:
    cyagen [OPTIONS] --source <SOURCE>

OPTIONS:
    -s, --source <SOURCE>               source file path
    -t, --temp-dir <TEMP_DIR>           template directory
    -o, --output-dir <OUTPUT_DIR>       output directory
    -j, --json-filepath <JSON_FILEPATH> output json file path
    -h, --help                          Print help information
    -V, --version                       Print version information
$
```