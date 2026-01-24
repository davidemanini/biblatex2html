

use minijinja::{context, Environment};

use biblatex::{Bibliography, Chunk, Date, DateValue, Entry, PermissiveType, Person, Spanned};
//use core::fmt;
//use std::fmt::{Display, Formatter};
//use serde::de::Error;
//use std::any::Any;
use std::io::{Write, stdout};
use std::{fs, io::Read};
//use std::fs::;
//use std::fmt::Display;
use std::error;
//e std::
use serde::{Serialize, Deserialize};
use regex::Regex;
use anyhow::{Context, Result};


use clap::{Parser};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CmdOptions{


    /// HTML Template to be used
    #[arg(long)]
    template: Option<std::path::PathBuf>,

    /// Bibtex file (default: stdin)
    #[arg(num_args(1))]
    input: Option<std::path::PathBuf>,

    /// Output file (default: stdout)
    #[arg(num_args(1))]
    output: Option<std::path::PathBuf>,

    /// Print the default template on the standard output
    #[arg(long)]
    print_template: bool,

}


impl CmdOptions {
    fn check(&self) -> Result<()> {
	if self.print_template
	    && (
		self.input.is_some()
		    || self.output.is_some()
		    || self.template.is_some()
	    ) {
		return Err(anyhow::Error::msg("Option --print-template must be used alone"));
	}
	Ok(())
    }
}

fn get_bibliography(content: &str) -> Result<Vec<BibEntry>> {
    let biblio = Bibliography::parse(&content).context("Fail to parse BibTeX")?;
    Ok(biblio.iter()
	.filter_map(|x| BibEntry::from_entry(x).ok())
	.collect::<Vec<_>>())
}
    

fn run_everithing(args: &CmdOptions) -> anyhow::Result<()> {

    args.check()?;

    let mut table = include_str!("table.html").to_string();


    if args.print_template {
	print!("{}", &table);
	return Ok(());
    }

    let mut input: Box<dyn Read> = match &args.input {
	Some(input) => Box::new(
	    fs::File::open(input)
		.with_context(|| format!("Fail to open file {:#?}", input))?
	),
	None => Box::new(std::io::stdin())
    };
    let mut content = String::new();
    input.read_to_string(&mut content)
	.with_context(|| format!("Fail to read input file {:#?}", args.input))?;


    let entries = get_bibliography(&content)?;

    if let Some(filename) = &args.template {
	table = "".to_string();
	fs::File::open(filename)
	    .with_context(|| format!("Fail to open template file {:#?}", filename))?
	    .read_to_string(&mut table)
	    .with_context(|| format!("Fail to read template file {:#?}", filename))?;
    }

    let html = build_html(&table, &entries)?;

    let mut output: Box<dyn Write> = match &args.output {
	Some(output) =>	Box::new(fs::File::create(output)
				 .with_context(|| format!("Fail to open file {:#?}", output))?
	),
	None => Box::new(stdout()),
    };
    
    output.write(html.as_bytes())
		.with_context(|| format!("Fail to write output file {:#?}", args.output))?;

    Ok(())
}



fn build_html(template: &str, entries: &Vec<BibEntry>) -> anyhow::Result<String> {
    let mut env = Environment::new();
    env.add_template("index.html", &template).context("Syntax error in the template")?;
    let template = env.get_template("index.html").unwrap();
    Ok(template.render(
	context!{biblio => entries}
    ).context("Fail to render template")?)
}



fn main() -> anyhow::Result<()> {

    let args = CmdOptions::parse();
    run_everithing(&args)
}


fn person_to_string(p: &Person) -> String{
    p.given_name.clone() + " " + &p.name
}

#[derive(Debug, Serialize, Deserialize)]
struct Author {
    pub name: String,
    pub surname: String,
}

impl Author {
    fn from_person(p: &Person) -> Self {
	Self{
	    name: p.given_name.clone(),
	    surname: p.name.clone()
	}
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct BibEntry{
    pub bib_type: String,
    pub key: String,
    pub author: String,
    pub author_list: Vec<Author>,
    pub title: String,
    pub url: Option<String>,
    pub doi: Option<String>,
    pub journal: Option<String>,
    pub date: Option<String>,
    pub file: Option<String>,
    pub note: Option<String>,
}


fn date_to_string(d: PermissiveType<Date>) -> Option<String> {
    match d {
	PermissiveType::Chunks(c) =>
	    Some(c.iter()
		 .map(|x| x.v.get())
		 .collect::<Vec<_>>()
		 .join("")),
	PermissiveType::Typed(date) =>
	    match date.value {
		DateValue::At(dd) => Some(dd.year.to_string()),
		_ => None
	    },
    }
}

fn jabref_file_parse(f: &str) -> Result<String, &str> {
    let re = Regex::new(r":(.+):PDF").unwrap();
    let res = re.captures(f);
    match res {
	Some(a) => Ok(a[1].to_string()),
	None => Err("Fail to parse jabref file field"),
    }
}

impl BibEntry{
    fn from_entry(e: &Entry) -> Result<Self, Box<dyn error::Error>> {
	Ok(Self{
	    bib_type: e.entry_type.to_string(),
	    key: e.key.clone(),
	    author_list: e.author()?.iter().map(|x| Author::from_person(x)).collect(),
	    author: e.author()?.iter()
		.map(|x| person_to_string(x))
		.collect::<Vec<_>>().join(", "),
	    title: e.title()?.iter()
		.map(|x| x.v.get())
		.collect::<Vec<_>>().join(""),
	    url: e.url().ok(),
	    doi: e.doi().ok(),
	    journal: e.journal().ok()
		.and_then(|x| Some(x.iter()
				   .map(|y| y.v.get())
				   .collect::<Vec<_>>()
				   .join(""))),
	    date: e.date().ok().and_then(|x| date_to_string(x)),
	    file: e.file().ok().and_then(|x| jabref_file_parse(&x).ok()),
	    note: e.note().ok()
		.and_then(|x| Some(chunks_to_str(x)))
/*	    note: e.note().ok()
		.and_then(|x| Some(x.iter()
				   .map(|y| y.v.get())
				   .collect::<Vec<_>>()
	    .join(""))),
	    */
	})
    }
}



fn chunks_to_str(c: &[Spanned<Chunk>]) -> String {
    c.iter().map(|y| y.v.get()).collect::<Vec<_>>().join("")
}
