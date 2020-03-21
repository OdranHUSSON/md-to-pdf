#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate log;

mod md_to_html;

use md_to_html::md_to_html;
use rocket::request::Form;
use rocket::response::{content, NamedFile};
use std::io::{Error, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

#[get("/")]
fn index() -> content::Html<String> {
    return content::Html(md_to_html(
        "
# md-to-pdf

An API service for converting markdown to PDF

## Routes

### POST /

Accepts markdown and responds with the converted PDF.

Send a form parameter `markdown` with the content to convert:

    curl -X POST -d 'markdown=# Heading 1' localhost:8000

You can also style the markdown through css:

    curl -X POST -d 'markdown=# Heading 1' -d 'css=h1 { color: red; }' localhost:8000
",
    ));
}

#[derive(FromForm)]
struct ConvertForm {
    markdown: String,
    css: Option<String>,
}

#[post("/", data = "<convert>")]
fn pandoc(convert: Form<ConvertForm>) -> Result<NamedFile, Error> {
    let stdin = Stdio::piped();
    let mut pandoc_builder = Command::new("pandoc");
    pandoc_builder
        .arg("--output=/tmp/markdown.pdf")
        .arg("--pdf-engine=wkhtmltopdf")
        .stdin(stdin);

    let mut css_file;
    if convert.css.is_some() {
        css_file = NamedTempFile::new()?;
        css_file.write_all(convert.css.as_ref().unwrap().as_bytes())?;
        pandoc_builder.arg("--css=".to_owned() + css_file.path().to_str().unwrap());
    }

    let mut pandoc_process = pandoc_builder.spawn()?;

    {
        let pandoc_stdin = pandoc_process.stdin.as_mut().unwrap();
        pandoc_stdin.write_all(convert.markdown.as_bytes())?;
    }

    let output = pandoc_process.wait_with_output()?;
    debug!("{:?}", output);

    NamedFile::open(Path::new("/tmp/markdown.pdf"))
}

fn main() {
    // Heroku compatibility
    let port_string = std::env::var("PORT");
    match port_string {
        Ok(p) => std::env::set_var("ROCKET_PORT", p),
        Err(_e) => (),
    }

    rocket::ignite().mount("/", routes![index, pandoc]).launch();
}
