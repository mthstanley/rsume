use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use tectonic::errors::Result;
use tectonic::{config, ctry, driver, status};
use tera::{Context, Tera};

use serde::{Deserialize, Deserializer, Serialize};
use toml::value::Datetime;

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
    highlights: Vec<String>,
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
    gpa: f64,
    achievements: Vec<String>,
    course: String,
    location: String,
    degree: String,
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
    summary: String,
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

fn main() {
    let path = Path::new("/Users/mtstanley/repos/rsume/resume.tex");
    // let content = fs::read_to_string(AsRef::<Path>::as_ref(path)).expect("unable to read file");

    let author: Author = toml::from_str(
        fs::read_to_string(Path::new("/Users/mtstanley/repos/rsume/author.toml"))
            .expect("couldn't read toml data file")
            .as_str(),
    )
    .expect("couldn't parse toml data");

    let tera = match Tera::new("templates/*") {
        Ok(t) => t,
        Err(e) => {
            print!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    let rendered = tera
        .render(
            "resume.tex",
            &Context::from_serialize(&author)
                .expect("couldn't convert author struct to tera context"),
        )
        .expect("rending template failed");

    let mut file = File::create("rendered.tex").expect("couldn't write rendered tex file");
    file.write_all(rendered.as_bytes())
        .expect("couldn't write rendered tex file");

    latex_to_pdf_2(path, rendered).expect("processing failed");
}

pub fn latex_to_pdf_2<P: AsRef<Path>>(latex_path: P, content: String) -> Result<()> {
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
            .filesystem_root(
                latex_path
                    .as_ref()
                    .parent()
                    .expect("filepath has no parent"),
            )
            .tex_input_name(
                latex_path
                    .as_ref()
                    .file_name()
                    .expect("filename was empty")
                    .to_str()
                    .expect("cannot convert osstring to string"),
            )
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(true)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(driver::OutputFormat::Pdf);

        let mut sess =
            ctry!(sb.create(&mut status); "failed to initialize the LaTeX processing session");
        ctry!(sess.run(&mut status); "the LaTeX engine failed");
        sess.into_file_data()
    };

    Result::Ok(())
}
