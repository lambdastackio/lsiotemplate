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

use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::fs::File;
use std::str::FromStr;
use std::cmp::Ord;

use term;
use rustc_serialize::json;
use rustc_serialize::json::{Json, ToJson};
//use rustc_serialize::base64::{STANDARD, ToBase64};
use std::collections::BTreeMap;

use clap::ArgMatches;
use handlebars::{Context, TemplateRenderError, JsonRender};

//use url::Url;
use yaml_rust::{YamlLoader, Yaml};
use lsio::error::Error;

use Client;
use Output;
use OutputFormat;
use Commands;
use TemplateTypes;
use DataTypes;


/// Commands
pub fn commands(matches: &ArgMatches, cmd: Commands, client: &mut Client) -> Result<(), Error>
{
    match cmd {
        Commands::json => {
            let result = json(matches, client);
        },
        Commands::yaml => {
            let result = yaml(matches, client);
        },
    }

    Ok(())
}

fn yaml(matches: &ArgMatches,
        client: &Client)
        -> Result<(), Error>
{
    match data_str(client) {
        Ok(data) => {
            // Convert from yaml to json
            let mut kv_btreemap = BTreeMap::new();

            match YamlLoader::load_from_str(&data) {
                Ok(yaml_vec) => {

                    let yaml = &yaml_vec[0];
                    match yaml.as_hash() {
                        Some(values) => {
                            for (key, value) in values.iter() {
                                let key_str = key.as_str().unwrap();
                                key_value(key_str.to_string(), &mut kv_btreemap, &value);
                            }
                        },
                        None => {},
                    }


                    let data_str = kv_btreemap.to_json();
                    process_output(data_str.to_string(), client)
                },
                Err(e) => {
                    println_color_quiet!(client.is_quiet, term::color::RED, "{:?}", e);
                    return Err(Error::from(Error::FileNotFound("yaml error".to_string())));
                },
            }
        },
        Err(e) => {
            println_color_quiet!(client.is_quiet, term::color::RED, "{:?}", e);
            return Err(e)
        },
    }
}

// Convert everything to String
fn key_value(key: String, mut bt: &mut BTreeMap<String, Json>, value: &Yaml) {
    match *value {
        Yaml::Boolean(ref val) => {
            let value = format!("{}", val);
            bt.insert(key, value.to_json());
        },
        Yaml::Integer(ref val) => {
            let value = format!("{}", val);
            bt.insert(key, value.to_json());
        },
        Yaml::Real(ref val) => {
            let value = format!("{}", val);
            bt.insert(key, value.to_json());
        },
        Yaml::String(ref val) => {
            let value = format!("{}", val);
            bt.insert(key, value.to_json());
        },
        Yaml::Hash(ref values) => {
            let mut kv_btreemap = BTreeMap::new();

            for (key, value) in values.iter() {
                let key_str = key.as_str().unwrap();
                key_value(key_str.to_string(), &mut kv_btreemap, &value);
            }

            bt.insert(key, kv_btreemap.to_json());
        },
        Yaml::Array(ref vals) => {
            let mut varr: Vec<Json> = Vec::new();

            for value in vals {
                varr.push(value.as_str().unwrap().to_string().to_json());
            }

            bt.insert(key, varr.to_json());
        },
        _ => {},
    }
}

fn data_str(client: &Client) -> Result<String, Error> {
    let mut data_str: String;

    match client.data_type {
        DataTypes::String => { data_str = client.data.clone() },
        DataTypes::File => {
            let mut data_file: File;

            match File::open(&client.data) {
                Ok(file) => { data_file = file },
                Err(e) => {
                    println_color_quiet!(client.is_quiet, term::color::RED, "{:?}", e);
                    return Err(Error::from(Error::FileNotFound(client.template.clone())));
                },
            }

            data_str = String::new();
            try!(data_file.read_to_string(&mut data_str));
        },
    };

    Ok(data_str)
}

fn json(matches: &ArgMatches,
        client: &Client)
        -> Result<(), Error>
{
    match data_str(client) {
        Ok(data) => process_output(data, client),
        Err(e) => {
            return Err(e)
        },
    }
}

fn process_output(data_str: String,
                  client: &Client)
                  -> Result<(), Error>
{
    // NB: This is a critical step which the docs for handlebars doesn't mention. The Json::Object must be created
    // and passed into Context::wraps.
    let data_obj = Json::from_str(&data_str).unwrap();

    let mut data = Context::wraps(&data_obj);
    let mut output: File;

    match File::create(&client.output_file) {
        Ok(file) => { output = file; },
        Err(e) => {
            return Err(Error::from(Error::FileIO(e)));
        },
    }

    match client.template_type {
        TemplateTypes::File => {
            let mut template: File;

            match File::open(&client.template) {
                Ok(file) => { template = file; },
                Err(e) => {
                    println_color_quiet!(client.is_quiet, term::color::RED, "{:?}", e);
                    return Err(Error::from(Error::FileNotFound(client.template.clone())));
                },
            }

            match client.handlebars.template_renderw2(&mut template, &data, &mut output) {
                Ok(_) => {},
                Err(e) => {
                    println_color_quiet!(client.is_quiet, term::color::RED, "{:?}", e);
                    return Err(Error::from(Error::FileNotFound("output".to_string())));
                },
            }
        },
        _ => {
            let mut template = client.template.clone();

            match client.handlebars.template_renderw(&mut template, &data, &mut output) {
                Ok(_) => {},
                Err(e) => {
                    println_color_quiet!(client.is_quiet, term::color::RED, "{:?}", e);
                    return Err(Error::from(Error::FileNotFound("output".to_string())));
                },
            }
        },
    }

    Ok(())
}
