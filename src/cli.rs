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

use clap::{App, Arg, SubCommand};


pub fn build_cli<'a>(app: &str, home: &'a str, version: &'a str) -> App<'a, 'a> {
  App::new(app)
    .about("Handlebars template utility. Pass in string (url format), json or yaml file as data input along with input/output file names")
    .author("Chris Jones")
    .version(version)
    .after_help("For more information about a specific command, try `lsiotemplate <command> --help`\nSource code for lsiotemplate available at: https://github.com/lambdastackio/lsiotemplate")
    .arg(Arg::with_name("config")
       .short("c")
       .long("config")
       .value_name("FILE")
       .default_value(home)
       .help("Sets a custom config file.")
       .takes_value(true))
    .arg(Arg::with_name("data")
       .short("d")
       .long("data")
       .value_name("data in json format")
       .help("pass in data in json format")
       .takes_value(true))
    /*
    .arg(Arg::with_name("format")
       .short("f")
       .long("format")
       .value_name("format of data")
       .help("Data format (json or yaml)")
       .default_value("json")
       .takes_value(true))
    */
    .arg(Arg::with_name("input")
       .short("i")
       .long("input")
       .value_name("template input string")
       .help("pass in a template input string instead of a file name")
       .takes_value(true))
    .arg(Arg::with_name("output")
       .short("o")
       .long("output")
       .value_name("FILE")
       .help("Path to output file. If not specified then output will be to stdout by default")
       .takes_value(true))
    .arg(Arg::with_name("file")
       .short("f")
       .long("file")
       .value_name("FILE")
       .help("Path to data input file. Data (-d) will supercede this option if present")
       .takes_value(true))
    .arg(Arg::with_name("template")
       .short("t")
       .long("template")
       .value_name("FILE")
       .help("Path to template input file. Input (-i) will supercede this option if present")
       .takes_value(true))
    .arg(Arg::with_name("output-format")
       .short("u")
       .long("output-format")
       .default_value("file")
       .value_name("file, pretty-json, json, plain or serialize")
       .help("Specifies the output to stdout (and disk in some cases). Options are file, json, none, noneall, pretty-json, plain, serialize")
       .takes_value(true))
    .arg(Arg::with_name("generate-bash-completions")
       .short("g")
       .long("generate-bash-completions")
       .help("Outputs bash completions"))
    .arg(Arg::with_name("proxy")
       .short("p")
       .long("proxy")
       .value_name("URL:<port>")
       .help("Sets a custom proxy URL:<port>. Default is to use http(s)_proxy and no_proxy for URL data request only")
       .takes_value(true))
    .arg(Arg::with_name("output-color")
       .short("l")
       .long("output-color")
       .default_value("green")
       .value_name("green or red or blue or yellow or white or normal")
       .help("Specifies the output color.")
       .takes_value(true))
    .arg(Arg::with_name("quiet")
       .short("q")
       .long("quiet")
       .help("No output to stdout is produced"))
    .subcommand(SubCommand::with_name("json")
       .about("json format of input data"))
    .subcommand(SubCommand::with_name("yaml")
       .about("yaml format of input data"))
}
