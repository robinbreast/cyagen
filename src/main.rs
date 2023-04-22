use anyhow::{Context, Result};
use clap::Parser;
use cyagen;
use std::fs;
use std::path::{self, Path};
use tera;
use serde_json;

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
    let parser = cyagen::Parser::parse(&code);
    if let Some(json_filepath) = args.json_filepath {
        let dirpath = Path::new(&json_filepath)
            .parent()
            .unwrap_or_else(|| Path::new("./"));
        if !dirpath.exists() {
            fs::create_dir_all(&dirpath).with_context(|| {
                format!("failed to create folder `{}`", dirpath.to_string_lossy())
            })?;
        }
        let mut output_fname = format!("{}", &json_filepath);
        if output_fname.contains("@sourcename@") {
            output_fname = output_fname.replace("@sourcename@", &sourcename);
        }
        cyagen::generate_json(&parser, &output_fname)?;
    } else if let (Some(output_dir), Some(temp_dir)) = (args.output_dir, args.temp_dir) {
        if Path::new(&output_dir).exists() {
            println!(
                "`{}` is already existed. select other folder as output",
                output_dir
            );
        } else {
            fs::create_dir_all(&output_dir)
                .with_context(|| format!("failed to create folder `{}`", output_dir))?;
            let temp_dir = fs::read_dir(&temp_dir)
                .with_context(|| format!("failed to read folder `{}`", temp_dir))?;
            for temp_file in temp_dir {
                let temp_path = temp_file.unwrap().path().to_string_lossy().into_owned();
                let temp_filename = temp_path.split(path::MAIN_SEPARATOR).last().unwrap();
                let temp = fs::read_to_string(&temp_path)
                    .with_context(|| format!("failed to read file `{}`", &temp_path))?;
                let temp_ext = Path::new(temp_filename)
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or("");
                let gen;
                let mut output_fname =
                    format!("{}{}{}", &output_dir, path::MAIN_SEPARATOR, temp_filename);
                if temp_ext == "tera" || temp_ext == "j2" {
                    output_fname = output_fname.replace(".tera", "").replace(".j2", "");
                    let mut tera = tera::Tera::default();
                    tera.add_raw_template("temp", &temp).unwrap();
                    let mut context = tera::Context::new();
                    let json_data = serde_json::to_string(&parser).unwrap();
                    let data: tera::Value = serde_json::from_str(&json_data).unwrap();
                    for (key, value) in data.as_object().unwrap() {
                        context.insert(key, value);
                    }
                    context.insert("sourcename", &sourcename);
                    let result = tera.render("temp", &context);
                    match result {
                        Ok(value) => gen = value,
                        Err(error) => panic!("Error: {}", error),
                    };
                } else {
                    gen = cyagen::generate(&parser, &temp, &sourcename);
                }
                if output_fname.contains("@sourcename@") {
                    output_fname = output_fname.replace("@sourcename@", &sourcename);
                }
                println!("generating ... {}", &output_fname);
                fs::write(&output_fname, gen)
                    .with_context(|| format!("failed to write file `{}`", &output_fname))?;
            }
            println!("done!");
        }
    } else {
        println!("wrong arguments given; you can generate json file or files based on templates at a time");
    }
    Ok(())
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
