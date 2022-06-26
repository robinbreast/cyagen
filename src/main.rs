use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path;

/// text file generator based on C file and templates
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// source file path
    #[clap(short, long)]
    source: String,

    /// template directory
    #[clap(short, long)]
    temp_dir: String,
    /// output directory
    #[clap(short, long)]
    output_dir: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if path::Path::new(&args.output_dir).exists() {
        println!(
            "`{}` is already existed. select other folder as output",
            args.output_dir
        );
    } else {
        fs::create_dir_all(&args.output_dir)
            .with_context(|| format!("failed to create folder `{}`", args.output_dir))?;
        let code = fs::read_to_string(&args.source)
            .with_context(|| format!("failed to open file `{}`", args.source))?;
        let temp_dir = fs::read_dir(&args.temp_dir)
            .with_context(|| format!("failed to read folder `{}`", args.temp_dir))?;
        let sourcename = path::Path::new(&args.source).with_extension("");
        let sourcename = sourcename.file_name().unwrap().to_str().unwrap().clone();
        let parser = cyagen::Parser::parse(&code);
        for temp_file in temp_dir {
            let temp_path = temp_file.unwrap().path().to_string_lossy().into_owned();
            let temp_filename = temp_path.split(path::MAIN_SEPARATOR).last().unwrap();
            let temp = fs::read_to_string(&temp_path)
                .with_context(|| format!("failed to read file `{}`", &temp_path))?;
            let gen = cyagen::generate(&parser, &temp, &sourcename);
            let mut output_fname = format!(
                "{}{}{}",
                &args.output_dir,
                path::MAIN_SEPARATOR,
                temp_filename
            );
            if output_fname.contains("@sourcename@") {
                output_fname = output_fname.replace("@sourcename@", &sourcename);
            }
            println!("generating ... {}", &output_fname);
            fs::write(&output_fname, gen)
                .with_context(|| format!("failed to write file `{}`", &output_fname))?;
        }
        println!("done!");
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
