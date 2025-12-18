

use minijinja::{context, Environment, Error};

use biblatex::{Bibliography, Entry, Person, PermissiveType, Date, DateValue};
//use std::any::Any;
use std::io::Write;
use std::{fs, io::Read};
//use std::fs::;
//use std::fmt::Display;
use std::error;
use serde::{Serialize, Deserialize};
use regex::Regex;

//use self::CmdOptions;


use clap::Parser;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CmdOptions{


    /// HTML Template to be used
    #[arg(long)]
    template: Option<String>,

    /// Incorporate Javascript inside the template
    #[arg(long)]
    embed_js: bool,

    /// Javascript file to be used
    #[arg(long)]
    js_file: Option<String>,

    /// Bibtex file
    #[arg(long)]
    bibtex: String,

    /// Output file
    #[arg(num_args(1))]
    output: String,
}


fn main() {

    let args = CmdOptions::parse();


    let output = args.output;
    
    let mut input = fs::File::open(args.bibtex).unwrap();


    let mut content = String::new();
    input.read_to_string(&mut content).unwrap();

    let biblio = Bibliography::parse(&content).unwrap();

    

    
/*
    for i in biblio {
	let authors: Vec<String> = i.author().unwrap().iter().map(|x| person_to_string(x)).collect();
	let auth = authors.join(", ");

	let title = i.title().unwrap().iter().map(|x| x.v.get()).collect::<Vec<_>>().join("");
	println!("{}: {}", auth, title);
	
}
    */

    let entries: Vec<BibEntry> = biblio.iter()
	.filter_map(|x| BibEntry::from_entry(x).ok())
	.collect();

    dbg!(entries.len());
    
    for i in &entries [0..entries.len().min(4)] {
	println!("{:?}\n", i)
    }

//    s = entries[0].serialize(true).unwrap(

    let mut env = Environment::new();
    let mut table = include_str!("table.html").to_string();
    if let Some(filename) = args.template {
	let mut f = fs::File::open(filename).unwrap();
	f.read_to_string(&mut table).unwrap();
    }


    
    
    env.add_template("index.html", &table).unwrap();
    let template = env.get_template("index.html").unwrap();
    let html = template.render(
	context!{biblio => entries}
    ).unwrap();

    let mut output = fs::File::create(output).unwrap();
    output.write(html.as_bytes()).unwrap();
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
//#[derive(Display)]
//#[display(fmt = "({}, {})", x, y)]
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
//    pub localfile: Option<String>,
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
	    date: date_to_string(e.date().unwrap()),
	    file: e.file().ok().and_then(|x| jabref_file_parse(&x).ok()),
	    note: e.note().ok()
		.and_then(|x| Some(x.iter()
				   .map(|y| y.v.get())
				   .collect::<Vec<_>>()
				   .join(""))),
	    
	})
    }
}


    
