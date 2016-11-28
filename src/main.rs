// Copyright 2016 LambdaStack All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

// NOTE: This attribute only needs to be set once.
#![doc(html_logo_url = "https://lambdastackio.github.io/static/images/lambdastack-200x200.png",
       html_favicon_url = "https://lambdastackio.github.io/static/images/favicon.ico",
       html_root_url = "https://lambdastackio.github.io/lsiotemplate/lsiotemplate/index.html")]

extern crate handlebars;
#[macro_use]
extern crate lsio;
extern crate rustc_serialize;
extern crate env_logger;
extern crate term;
extern crate url;
#[macro_use]
extern crate clap;
extern crate yaml_rust;
extern crate ini;

use std::io::{self, Write};
use std::env;
use std::path::PathBuf;
use std::convert::AsRef;

//use rustc_serialize::json::Json;
use handlebars::Handlebars;
use clap::Shell;

use lsio::config::ConfigFile;
use lsio::error::Error;

mod cli;
mod commands;

/// Allows you to set the output type for stderr and stdout.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    File,
    JSON,
    None,
    NoneAll, // NoneAll is the same as None but will also not write out objects to disk
    PrettyJSON,
    Plain,
    Serialize,
}

/// Allows you to control Error output.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErrorOutput {
    /// Defaults to OutputFormat::serialize since it's easier to debug.
    ///
    /// Available formats are json, plain, serialize or none (don't output anything)
    pub format: OutputFormat,
    /// Can be any term color. Defaults to term::color::RED.
    pub color: term::color::Color,
}

/// Allows you to control non-Error output.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output {
    /// Defaults to OutputFormat::plain.
    ///
    /// Available formats are json, plain, serialize or none (don't output anything).
    /// If plain is used then you can serialize structures with format! and then pass the output.
    pub format: OutputFormat,
    /// Can be any term color. Defaults to term::color::GREEN.
    pub color: term::color::Color,
}

/// Commands
///
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Commands {
    json,
    yaml,
    ini,
}

/// TemplateTypes
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateTypes {
    String,
    File,
}

/// DataTypes - Data is either input via string or file
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataTypes {
    String,
    File,
}

//pub struct Client<'a>
/// Client struct that will carry options around.
pub struct Client
{
    //pub config: &'a mut config::Config,
    pub handlebars: Handlebars,
    pub data: String,
    pub data_type: DataTypes,
    pub template: String,
    pub template_type: TemplateTypes,
    pub output_file: String,
    pub error: ErrorOutput,
    pub output: Output,
    pub is_quiet: bool,
}

fn main() {
    let mut is_quiet = false;

    env_logger::init().unwrap();

    let version = format!("{}", crate_version!());
    let mut home: PathBuf;
    // Get $HOME directory and set the default config. Let the CLI override the default.
    match env::home_dir() {
        Some(path) => {
            home = path;
            home.push(".lsiotemplate/config");
        },
        None => home = PathBuf::new(),
    }

    // NOTE: If the CLI info passed in does not meet the requirements then build_cli will panic!
    let matches = cli::build_cli("lsiotemplate", home.to_str().unwrap_or(""), &version).get_matches();

    if matches.is_present("generate-bash-completions") {
        cli::build_cli("lsiotemplate", home.to_str().unwrap_or(""), &version)
            .gen_completions_to("lsiotemplate", Shell::Bash, &mut io::stdout());
        ::std::process::exit(0);
    }

    // If the -q or --quiet flag was passed then shut off all output
    if matches.is_present("quiet") {
        is_quiet = true;
    }

    let data = matches.value_of("data").unwrap_or("");
    let data_file = matches.value_of("file").unwrap_or("");
    let proxy_str = matches.value_of("proxy");
    let output_file = matches.value_of("output").unwrap_or("");
    let input = matches.value_of("input").unwrap_or("");
    let template_file = matches.value_of("template").unwrap_or("");

    if data.is_empty() && data_file.is_empty() {
        println_color_quiet!(is_quiet, term::color::RED, "-d must be specified with valid json data or -f with a valid path to a data file.\n\n");
        println_color_quiet!(is_quiet, term::color::RED, "{}", matches.usage());
        ::std::process::exit(1);
    }

    if template_file.is_empty() && input.is_empty() {
        println_color_quiet!(is_quiet, term::color::RED, "-i or -t must be specified with a valid template string or template file path.\n\n");
        println_color_quiet!(is_quiet, term::color::RED, "{}", matches.usage());
        ::std::process::exit(1);
    }

    let output_format = match matches.value_of("output-format").unwrap_or("file").to_string().to_lowercase().as_ref() {
        "file" => OutputFormat::File,
        "json" => OutputFormat::JSON,
        "none" => OutputFormat::None,
        "noneall" => OutputFormat::NoneAll,
        "plain" => OutputFormat::Plain,
        _ => OutputFormat::PrettyJSON,
    };

    if output_format == OutputFormat::File && output_file.is_empty() {
        println_color_quiet!(is_quiet, term::color::RED, "-o was not specified with a valid output file.\n\n");
        println_color_quiet!(is_quiet, term::color::RED, "{}", matches.usage());
        ::std::process::exit(1);
    }

    let output_color = match matches.value_of("output-color").unwrap().to_string().to_lowercase().as_ref() {
        "red" => term::color::RED,
        "blue" => term::color::BLUE,
        "yellow" => term::color::YELLOW,
        "white" => term::color::WHITE,
        _ => term::color::GREEN,
    };

    // Set the config_file path to the default if a value is empty or set it to the passed in path value
    /*
    let config_option = matches.value_of("config").unwrap();

    let mut config_file: PathBuf;
    if config_option.is_empty() {
        config_file = home.clone();
    } else {
        config_file = PathBuf::new();
        config_file.push(config_option);
    }

    let mut config = config::Config::from_file(config_file).unwrap_or(config::Config::default());
    if proxy_str.is_some() {
        config.set_proxy(Some(Url::parse(proxy_str.unwrap()).unwrap()));
    }
    */

    let output = Output{format: output_format, color: output_color};

    let mut handlebars = Handlebars::new();

    let mut client = Client {
        //config: &mut config,
        handlebars: handlebars,
        data: if data.is_empty() {data_file.to_string()} else {data.to_string()},
        data_type: if data.is_empty() {DataTypes::File} else {DataTypes::String},
        template: if input.is_empty() {template_file.to_string()} else {input.to_string()},
        template_type: if input.is_empty() {TemplateTypes::File} else {TemplateTypes::String},
        output_file: output_file.to_string(),
        error: ErrorOutput {
            format: OutputFormat::Serialize,
            color: term::color::RED,
        },
        output: output,
        is_quiet: is_quiet,
    };

    let res = match matches.subcommand() {
        ("json", Some(sub_matches)) => commands::commands(sub_matches, Commands::json, &mut client),
        ("yaml", Some(sub_matches)) => commands::commands(sub_matches, Commands::yaml, &mut client),
        ("ini", Some(sub_matches)) => commands::commands(sub_matches, Commands::ini, &mut client),
        (e, _) => {
            let error = format!("{}", e);
            println_color_quiet!(client.is_quiet, term::color::RED, "{}\n\n", error);
            Err(Error::from(Error::CommandNotRecognized(error)))
        },
    };

    if let Err(e) = res {
        println_color_quiet!(client.is_quiet, term::color::RED, "{}", matches.usage());
        ::std::process::exit(1);
    }
}
