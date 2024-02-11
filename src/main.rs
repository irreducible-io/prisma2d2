use std::{
    io::{BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use psl::{parse_schema, schema_ast::ast::FieldType};

/// Visualize a Prisma schema as a d2 diagram.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Parse the Prisma schema from a file.
    /// Defaults to stdin.
    #[arg()]
    input_file: Option<PathBuf>,
    /// Write the d2 diagram to a file.
    /// Defaults to stdout.
    #[arg(short, long)]
    output_file: Option<PathBuf>,
}

fn input_reader(path: Option<PathBuf>) -> Result<Box<dyn Read>, std::io::Error> {
    if let Some(path) = path {
        let f = std::fs::File::open(path)?;
        Ok(Box::new(BufReader::new(f)))
    } else {
        Ok(Box::new(std::io::stdin()))
    }
}

fn read_input(read: &mut dyn Read) -> Result<String, std::io::Error> {
    let mut s = String::new();
    read.read_to_string(&mut s)?;
    Ok(s)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let input = read_input(&mut input_reader(args.input_file)?)?;
    let parsed = parse_schema(&input)?;
    for model in parsed.db.walk_models() {
        println!("{} {{\n\tshape: sql_table", model.database_name());
        for field in model.fields() {
            let f = field.ast_field();
            let t = match f.field_type {
                FieldType::Supported(ref i) => &i.name,
                FieldType::Unsupported(ref s, _) => s,
            };
            println!("\t{}: {}", field.name(), t);
        }
        println!("}}");
    }
    Ok(())
}
