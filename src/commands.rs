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
use ini::Ini;
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
        /// JSON - Handlebars uses JSON data to populate templates so all input must be converted
        /// to JSON if it's not already JSON.
        Commands::json => {
            let result = json(matches, client);
        },
        /// YAML - Is a human readable JSON so to speak. It's great for storing base data that
        /// feeds JSON based systems as long as you can convert it to JSON.
        Commands::yaml => {
            let result = yaml(matches, client);
        },
        /// INI - This is an older style configuration format but it's important to include it
        /// since a lot of systems still use it. INI has sections which can include `.` etc and
        /// keys that can have ` `. However, these things can cause issues with Handlebars so
        /// there are two special BTreeMaps optionally added to the output and they begin with `_`.
        /// _keys_replace and _sections_replace contain a key and value. The key represent the
        /// modified version of the original section name or key name. If they contain `.` or ` `
        /// then those values are replaced with `_` and inserted to either _keys_replace or
        /// _sections_replace depending where it belongs and then the original (unmodified)
        /// version of the name is added as the value so that they originals can be looked up
        /// if you need to add the original values of the key or section.
        /// If the key or section name does not contain an `.` or ` ` then it's skipped and if
        /// there are no modifications then the BTreeMaps are not added. 
        Commands::ini => {
            let result = ini(matches, client);
        },
    }

    Ok(())
}

fn ini(matches: &ArgMatches,
       client: &Client)
       -> Result<(), Error>
{
    match data_str(client) {
        Ok(data) => {
            // Convert from ini to json
            let mut kv_btreemap = BTreeMap::new();
            let mut _sections = BTreeMap::new();
            let mut _keys = BTreeMap::new();

            match Ini::load_from_str(&data) {
                Ok(ini) => {
                    // INI files do not have inner sections so they are only one level deep

                    for (sec, prop) in ini.iter() {
                        let section_name = sec.as_ref().unwrap().to_string();
                        let mut kv_prop = BTreeMap::new();

                        for (k, v) in prop.iter() {
                            // NOTE: Could make property that contains a `,` into an Array if desired
                            let new_key = k.clone();
                            // NB: The `underscore` Vec is an array of the original key value if
                            // the key contains ` `. Since Handlebars and other templating tools
                            // can not handle spaces they have to be replaced with `_`. To make
                            // sure keys that contain valid `_` are not replaced with ` ` on the
                            // the reverse side of this you must lookup the key to see if it exists
                            // and then take the original value if it does exists etc.
                            // It also uses the section since BTreeMaps replace the value of an
                            // existing key so we need to further tighten it.
                            if new_key.contains(" ") {
                                _keys.insert(format!("{}:{}", section_name.clone(), new_key.clone().replace(" ", "_")), new_key.clone());
                            }
                            kv_prop.insert(new_key.replace(" ", "_"), v.clone().to_json());
                        }

                        if section_name.contains(".") {
                            _sections.insert(section_name.clone().replace(".", "_"), section_name.clone());
                        }
                        kv_btreemap.insert(section_name.replace(".", "_"), kv_prop.to_json());

                    }

                    if _keys.len() > 0 {
                        kv_btreemap.insert("_key_replace".to_string(), _keys.to_json());
                    }
                    if _sections.len() > 0 {
                        kv_btreemap.insert("_section_replace".to_string(), _sections.to_json());
                    }

                    let data_str = kv_btreemap.to_json();

                    println!("{:#?}", data_str);

                    process_output(data_str.to_string(), client)
                },
                Err(e) => {
                    println_color_quiet!(client.is_quiet, term::color::RED, "{:?}", e);
                    return Err(Error::from(Error::FileNotFound("ini error".to_string())));
                },
            }
        },
        Err(e) => {
            println_color_quiet!(client.is_quiet, term::color::RED, "{:?}", e);
            Err(e)
        },
    }
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
                                yaml_key_value(key_str.to_string(), &mut kv_btreemap, &value);
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
            Err(e)
        },
    }
}

// Convert everything to String
fn yaml_key_value(key: String, mut bt: &mut BTreeMap<String, Json>, value: &Yaml) {
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
                yaml_key_value(key_str.to_string(), &mut kv_btreemap, &value);
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
