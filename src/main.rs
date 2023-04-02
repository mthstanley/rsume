use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::Parser;
use tectonic::errors::Result;
use tectonic::{config, ctry, driver, status};
use tera::{try_get_value, Context, Tera, Value};

use serde::{Deserialize, Deserializer, Serialize};
use toml::value::Datetime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(value_parser = parse_path)]
    input_path: PathBuf,
    #[arg(value_parser = parse_path)]
    template_path: PathBuf,
    template_filename: String,
    #[arg(value_parser = parse_path)]
    tex_root: PathBuf,
    #[arg(value_parser = parse_path)]
    output_root: PathBuf,
}

fn parse_path(s: &str) -> std::result::Result<PathBuf, String> {
    Ok(Path::new(s).to_path_buf())
}

#[derive(Serialize, Deserialize)]
struct Location {
    address: String,
    postal_code: String,
    city: String,
    country_code: String,
    region: String,
}

#[derive(Serialize, Deserialize)]
struct Social {
    username: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Company {
    name: String,
    location: String,
}

#[derive(Serialize, Deserialize)]
struct Experience {
    company: Company,
    department: String,
    position: String,
    website: String,
    #[serde(deserialize_with = "datetime_to_string")]
    start_date: String,
    #[serde(default)]
    #[serde(deserialize_with = "datetime_to_option_string")]
    end_date: Option<String>,
    current: bool,
    display: Vec<String>,
    highlights: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct GradePointAverage {
    major: f64,
    overall: f64,
}

#[derive(Serialize, Deserialize)]
struct Education {
    institution: String,
    website: String,
    major: String,
    minor: String,
    #[serde(deserialize_with = "datetime_to_string")]
    start_date: String,
    #[serde(default)]
    #[serde(deserialize_with = "datetime_to_option_string")]
    end_date: Option<String>,
    current: bool,
    gpa: GradePointAverage,
    achievements: Vec<String>,
    location: String,
    degree: String,
    latin_honors: String,
}

#[derive(Serialize, Deserialize)]
struct Skill {
    name: String,
    level: String,
    keywords: String,
    category: String,
}

#[derive(Serialize, Deserialize)]
struct Project {
    name: String,
    website: String,
    source: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct Author {
    name: String,
    email: String,
    description: String,
    picture: String,
    phone: String,
    website: String,
    location: Location,
    social: HashMap<String, Social>,
    experiences: Vec<Experience>,
    educations: Vec<Education>,
    skills: Vec<Skill>,
    projects: Vec<Project>,
}

fn datetime_to_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let datetime: Datetime = Deserialize::deserialize(deserializer)?;
    Ok(datetime.to_string())
}

fn datetime_to_option_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.map(|d: Datetime| d.to_string()))
}

fn escape_latex(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let input = try_get_value!("escape_latex", "value", String, value);
    let mut output = String::with_capacity(input.len() * 2);
    for c in input.chars() {
        match c {
            '&' | '%' | '#' | '$' => output.push_str(format!("\\{}", c).as_str()),
            _ => output.push(c),
        }
    }

    Ok(Value::String(output))
}

fn main() {
    let args = Args::parse();

    let author: Author = toml::from_str(
        fs::read_to_string(args.input_path)
            .expect("couldn't read toml data file")
            .as_str(),
    )
    .expect("couldn't parse toml data");

    let mut tera = match Tera::new(
        args.template_path
            .to_str()
            .expect("Template path must be present"),
    ) {
        Ok(t) => t,
        Err(e) => {
            print!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.register_filter("escape_latex", escape_latex);

    let rendered = tera
        .render(
            &args.template_filename,
            &Context::from_serialize(&author)
                .expect("couldn't convert author struct to tera context"),
        )
        .expect("rending template failed");

    // File::create(Path::new("rendered.tex"))
    //     .expect("cannot create file")
    //     .write_all(rendered.as_bytes())
    //     .expect("failed to write rendered template");

    latex_to_pdf(
        args.tex_root,
        args.template_filename,
        rendered,
        args.output_root,
    )
    .expect("processing failed");
}

pub fn latex_to_pdf(
    tex_root: PathBuf,
    tex_filename: String,
    content: String,
    output_root: PathBuf,
) -> Result<()> {
    let mut status = status::NoopStatusBackend::default();

    let auto_create_config_file = false;
    let config = ctry!(config::PersistentConfig::open(auto_create_config_file);
                       "failed to open the default configuration file");

    let only_cached = false;
    let bundle = ctry!(config.default_bundle(only_cached, &mut status);
                       "failed to load the default resource bundle");

    let format_cache_path = ctry!(config.format_cache_path();
                                  "failed to set up the format cache");

    {
        // Looking forward to non-lexical lifetimes!
        let mut sb = driver::ProcessingSessionBuilder::default();
        sb.bundle(bundle)
            .primary_input_buffer(content.as_bytes())
            .filesystem_root(tex_root.as_path().parent().expect("filepath has no parent"))
            .tex_input_name(&tex_filename)
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(driver::OutputFormat::Pdf)
            .output_dir(output_root);

        let mut sess =
            ctry!(sb.create(&mut status); "failed to initialize the LaTeX processing session");
        ctry!(sess.run(&mut status); "the LaTeX engine failed");
        sess.into_file_data()
    };

    Result::Ok(())
}
