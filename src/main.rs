use anyhow::{Context, Result};
use clap::Parser;
use cyagen;
use std::fs;
use std::path::{self, Component, Path, PathBuf};

/// text file generator based on C file and templates
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// source file path
    #[arg(short, long)]
    source: String,
    /// template directory
    #[arg(short, long)]
    temp_dir: Option<String>,
    /// output directory
    #[clap(short, long)]
    output_dir: Option<String>,
    /// output json file path
    #[clap[short, long]]
    json_filepath: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let code = fs::read_to_string(&args.source)
        .with_context(|| format!("failed to open file `{}`", args.source))?;
    let sourcename = Path::new(&args.source).with_extension("");
    let sourcename = sourcename.file_name().unwrap().to_str().unwrap().clone();
    // parse a C file
    let mut parser: cyagen::Parser = cyagen::Parser::parse(&code);
    parser.sourcename = sourcename.to_string();
    // check if json filepath specified as output
    if let Some(json_filepath) = args.json_filepath {
        let dirpath = Path::new(&json_filepath)
            .parent()
            .unwrap_or_else(|| Path::new("./"));
        if !dirpath.exists() {
            fs::create_dir_all(&dirpath).with_context(|| {
                format!("failed to create folder `{}`", dirpath.to_string_lossy())
            })?;
        }
        let sourcedirname = get_relative_path(&dirpath.to_string_lossy(), &args.source).unwrap();
        let sourcedirname = Path::new(&sourcedirname)
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        parser.sourcedirname = sourcedirname;
        let mut output_fname = format!("{}", &json_filepath);
        if output_fname.contains("@sourcename@") {
            output_fname = output_fname.replace("@sourcename@", &sourcename);
        }
        cyagen::generate_json(&parser, &output_fname)?;
    // check if ouput filepath specified as output
    } else if let (Some(output_dir), Some(temp_dir)) = (args.output_dir, args.temp_dir) {
        let sourcedirname = get_relative_path(&output_dir, &args.source).unwrap();
        let sourcedirname = Path::new(&sourcedirname)
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        parser.sourcedirname = sourcedirname;
        let _ = generate_files(&parser, Path::new(&temp_dir), Path::new(&output_dir));
        println!("done!");
    } else {
        println!("wrong arguments given; you can generate json file or files based on templates at a time");
    }
    Ok(())
}

fn generate_files(parser: &cyagen::Parser, temp_dir: &Path, output_dir: &Path) -> Result<()> {
    let mut result = Ok(());
    if !Path::new(&output_dir).exists() {
        fs::create_dir_all(&output_dir)
            .with_context(|| format!("failed to create folder `{}`", output_dir.display()))?;
    }
    let temp_dir = fs::read_dir(&temp_dir)
        .with_context(|| format!("failed to read folder `{}`", temp_dir.display()))?;
    for entry in temp_dir {
        let entry = entry?;
        let path = entry.path();
        let is_file = path.is_file();
        let temp_path = path.into_os_string().into_string().unwrap();
        let temp_filename = temp_path.split(path::MAIN_SEPARATOR).last().unwrap();
        if is_file {
            let temp = fs::read_to_string(&temp_path)
                .with_context(|| format!("failed to read file `{}`", &temp_path))?;
            let temp_ext = Path::new(temp_filename)
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("");
            let mut gen;
            let mut output_fname = format!(
                "{}{}{}",
                output_dir.display(),
                path::MAIN_SEPARATOR,
                temp_filename
            );
            if output_fname.contains("@sourcename@") {
                output_fname = output_fname.replace("@sourcename@", &parser.sourcename);
            }
            output_fname = output_fname
                .replace(".tera", "")
                .replace(".j2", "")
                .replace(".njk", "");
            println!("rendering ... {}", &output_fname);
            // check if template format is jinja2 such as .tera, .j2, or .njk
            if temp_ext == "tera" || temp_ext == "j2" || temp_ext == "njk" {
                gen = cyagen::generate_using_tera(&parser, &temp);
            // check if cyagen old style of template format
            } else {
                gen = cyagen::generate(&parser, &temp, &parser.sourcename);
            }
            // check if output file is already existed, then merge with manual sections
            if PathBuf::from(&output_fname).exists() {
                let old_gen = fs::read_to_string(&output_fname)
                    .with_context(|| format!("failed to read file `{}`", &output_fname))?;
                gen = cyagen::merge_with_manual_sections(&gen, &old_gen);
            }
            //
            fs::write(&output_fname, gen)
                .with_context(|| format!("failed to write file `{}`", &output_fname))?;
        } else {
            let output_path =
                output_dir.join(&temp_filename.replace("@sourcename@", &parser.sourcename));
            result = generate_files(&parser, Path::new(&temp_path), &output_path);
        }
    }
    result
}

fn get_relative_path(from_pathstr: &str, to_pathstr: &str) -> Option<String> {
    let to_path = Path::new(to_pathstr).to_path_buf();
    let from_path = Path::new(from_pathstr).to_path_buf();

    let common_prefix = to_path
        .components()
        .zip(from_path.components())
        .take_while(|(a, b)| a == b)
        .count();

    let up_levels = from_path.components().count() - common_prefix;

    let relative_path = std::iter::repeat(Component::ParentDir)
        .take(up_levels)
        .chain(to_path.components().skip(common_prefix))
        .collect::<PathBuf>();

    relative_path.to_str().map(String::from)
}


#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn test_generate() {
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
@static-vars@@dtype@ @name@;
@end-static-vars@
// functions
@fncs@@rtype@ @name@(@args@);
@end-fncs@
";
        let expected = "\
// include
#include <stdio.h>

// local variables
int var;

// functions
int func1();
int func2(char c);

";
        let parser = cyagen::Parser::parse(code);
        let gen = cyagen::generate(&parser, temp, sourcename);
        assert_eq!(gen, expected);
    }
}
