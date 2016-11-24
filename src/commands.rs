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

use term;
use rustc_serialize::json;
use rustc_serialize::json::{Json, ToJson};
//use rustc_serialize::base64::{STANDARD, ToBase64};
use std::collections::BTreeMap;

use clap::ArgMatches;
use handlebars::{Context, TemplateRenderError, JsonRender};

//use url::Url;
use lsio::error::Error;

use Client;
use Output;
use OutputFormat;
use Commands;
use TemplateTypes;

/// Commands
pub fn commands(matches: &ArgMatches, cmd: Commands, client: &mut Client) -> Result<(), Error>
{
    match cmd {
        Commands::json => {
            let result = json(matches, client);
            /*
            match matches.subcommand() {
                ("raw", Some(matches)) => {
                    let result = raw(matches, client);
                },
                ("json", Some(matches)) => {

                },
                ("url", Some(matches)) => {

                },
                ("toml", Some(matches)) => {

                },
                ("yaml", Some(matches)) => {

                },
                (_,_) => {

                },
            }
            */
        },
    }

    Ok(())
}

fn json(matches: &ArgMatches,
        client: &Client)
        -> Result<(), Error>
{
    let data_str: String = client.data.clone();
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
                    return Err(Error::from(Error::FileNotFound(client.template.clone())));
                },
            }

            match client.handlebars.template_renderw2(&mut template, &data, &mut output) {
                Ok(_) => {},
                Err(e) => {
                    return Err(Error::from(Error::FileNotFound("output".to_string())));
                },
            }
        },
        _ => {
            let mut template = client.template.clone();

            match client.handlebars.template_renderw(&mut template, &data, &mut output) {
                Ok(_) => {},
                Err(e) => {
                    return Err(Error::from(Error::FileNotFound("output".to_string())));
                },
            }
        },
    }

    Ok(())
}
