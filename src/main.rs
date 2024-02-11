use std::{
    fmt::Display,
    io::{BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use psl::{
    parse_schema,
    parser_database::walkers::Walker,
    schema_ast::ast::{FieldId, FieldType, ModelId},
    ValidatedSchema,
};

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

struct D2Diagram {
    sql_tables: Vec<D2SqlTable>,
    relations: Vec<D2Relation>,
}

impl D2Diagram {
    fn new() -> Self {
        D2Diagram {
            sql_tables: vec![],
            relations: vec![],
        }
    }
}

struct D2SqlTable {
    name: String,
    columns: Vec<D2SqlColumn>,
}

impl D2SqlTable {
    fn with_name(name: String) -> Self {
        D2SqlTable {
            name,
            columns: vec![],
        }
    }
}

struct D2SqlColumn {
    name: String,
    datatype: String,
    constraints: Vec<SqlConstraint>,
}

impl D2SqlColumn {
    fn with_name_and_datatype(name: String, datatype: String) -> Self {
        D2SqlColumn {
            name,
            datatype,
            constraints: vec![],
        }
    }
}

enum SqlConstraint {
    PrimaryKey,
    ForeignKey,
    Unique,
}

struct D2Relation {
    from: String,
    to: String,
    label: Option<String>,
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
    let diagram = render(&parsed);
    println!("{}", diagram);
    Ok(())
}

fn render(schema: &ValidatedSchema) -> D2Diagram {
    let mut diagram = D2Diagram::new();
    for model in schema.db.walk_models() {
        let (table, mut relations) = render_model(model);
        diagram.sql_tables.push(table);
        diagram.relations.append(&mut relations);
    }
    diagram
}

fn render_model(model: Walker<'_, ModelId>) -> (D2SqlTable, Vec<D2Relation>) {
    let mut table = D2SqlTable::with_name(model.name().to_owned());
    let mut relations = vec![];
    for field in model.fields() {
        let (column, mut r) = render_field(model.name(), field);
        table.columns.push(column);
        relations.append(&mut r);
    }
    (table, relations)
}

fn render_field(
    table_name: &str,
    field: Walker<'_, (ModelId, FieldId)>,
) -> (D2SqlColumn, Vec<D2Relation>) {
    let f = field.ast_field();
    let t = match f.field_type {
        FieldType::Supported(ref i) => &i.name,
        FieldType::Unsupported(ref s, _) => s,
    };
    let mut column = D2SqlColumn::with_name_and_datatype(field.name().to_owned(), t.to_owned());
    let mut relations = vec![];
    for attr in &f.attributes {
        if attr.name.name == "id" {
            column.constraints.push(SqlConstraint::PrimaryKey);
        } else if attr.name.name == "unique" {
            column.constraints.push(SqlConstraint::Unique);
        } else if attr.name.name == "relation" {
            relations.push(D2Relation {
                from: table_name.to_owned(),
                to: t.to_owned(),
                label: None,
            })
        }
    }
    (column, relations)
}

impl Display for D2Diagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for table in &self.sql_tables {
            write!(f, "{}\n\n", table)?;
        }
        for relation in &self.relations {
            write!(f, "{}\n\n", relation)?;
        }
        Ok(())
    }
}

impl Display for D2SqlTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{\n", self.name)?;
        write!(f, "\tshape: sql_table\n")?;
        for column in &self.columns {
            write!(f, "\t{}: {}", column.name, column.datatype)?;
            if !column.constraints.is_empty() {
                write!(
                    f,
                    " {{ constraint: [{}] }}",
                    column
                        .constraints
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                )?;
            }
            write!(f, "\n")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl Display for SqlConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqlConstraint::ForeignKey => write!(f, "foreign_key"),
            SqlConstraint::PrimaryKey => write!(f, "primary_key"),
            SqlConstraint::Unique => write!(f, "unique"),
        }
    }
}

impl Display for D2Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)?;
        if let Some(ref label) = self.label {
            write!(f, ": {}", label)?;
        }
        Ok(())
    }
}
