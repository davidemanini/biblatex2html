# biblatex2html

## About

Convert a bibliography file in BibTeX format into a HTML webpage, using a given [Jinja](https://jinja.palletsprojects.com/en/stable/) template.

The default template generates a table with a line for each entry of the bibliography.  It is possible to show/hide certain columns.  You can also filter by author(s).

## Installation

Compile using Cargo:

```bash
sudo apt-get install cargo
cargo build --release
```

Then the file `target/release/biblatex2html` is a statically-linked executable with no external dependency.

## Usage

```bash
biblatex2html input.bib output.html
```

If you don't like the default template, you can specify a custom template:

```bash
biblatex2html --template template.html input.bib output.html
```

## Creating a custom template

Templates are specified using the [Jinja](https://jinja.palletsprojects.com/en/stable/) syntax.

The program exposes the vector `entries`.  Each element of `entries` is a dictionary, whose keys are `title`, `author`, `year`, and so on.

To see an example of a template, you can print the default template:

```bash
biblatex2html --print-template
```

## Roadmap/Future developments

 * Take several input file
 * Add more columns
 * Improve the search feature:
   * Search on different fields
   * Fuzzy search
   * Regex?
   * ...
 * Sort by column
 * Add a sane CSS
 * Apply global filters
 * ...
 
## Comments

Feedback is welcome!

## Similar project(s)

* [bibtex2html](https://usr.lmf.cnrs.fr/~jcf/bibtex2html/index.en.html)
  Same purpose.  Instead of using Jinja templates, it uses .bst (BibTeX style) files.
