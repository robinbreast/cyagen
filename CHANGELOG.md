# Change Log

## [0.1.10]
### New 
- supports *.njk extension for template file
- supports MANUAL SECTION; refer to gtest template files
### Fixed
- fix patterns for **fncs** and **static_vars**
### Updated
- **gtest** template files are updated with ones from [vscode-cyagen](https://marketplace.visualstudio.com/items?itemName=robinbreast.vscode-cyagen&ssr=false#overview)
- refactored to separate modules for Parser and generate functions

## [0.1.9]
### New 
supports jinja2 template files (*.j2 or *.tera) using tera engine

## [0.1.8]
 
### New 
new option --json-filepath to generate json file; to use another better template engine like jinja2
 
## [0.1.5]
 
refactored on **@local-vars@** tag by introducing new tags
 
### New 
- new tags: **@static-vars@**, **@static-global-vars@**, and **@static-local-vars@** for static variables which are declared in a function
- new tags: **@name-expr@**
 
### Changed
- **@local-vars@** had been removed and replaced with new **@static-vars@** tag
- **@name@** in **@static-[global|local]-vars@** contains only the variable name string; you can use **@name-expr@** tag in order to have the full string of data variable name with brackets in case when the variable is array data type
 
### Fixed
- none 