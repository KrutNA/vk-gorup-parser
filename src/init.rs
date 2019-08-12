use clap::{App, Arg};
use try_from::TryFrom;
use yaml_rust::{YamlLoader, Yaml};
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SearchArg
{
    pub group_id: i64,
    pub pattern: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Value
{
    Str(String),
    Vec(Vec<SearchArg>),
    Int(u64),
}
pub trait Convert
{
    fn as_u64(&self) -> Result<u64,()>;
    fn as_str(&self) -> Result<String,()>;
    fn as_vec(&self) -> Result<&Vec<SearchArg>,()>;
    fn as_mut_vec(&mut self) -> Result<&mut Vec<SearchArg>,()>;
} 

impl Convert for Value
{
    fn as_u64(&self) -> Result<u64,()> {
        match self {
            Value::Int(v) => Ok(*v),
            _ => Err(()),
        }
    }
    fn as_str(&self) -> Result<String,()> {
        match self {
            Value::Str(v) => Ok(v.clone()),
            _ => Err(()),
        }
    } 
    fn as_vec(&self) -> Result<&Vec<SearchArg>,()> {
        match *self {
            Value::Vec(ref v) => Ok(v),
            _ => Err(()),
        }
    }
    fn as_mut_vec(&mut self) -> Result<&mut Vec<SearchArg>,()> {
        match *self {
            Value::Vec(ref mut v) => Ok(v),
            _ => Err(()),
        }
    }
}



pub fn initialize(
    map: &mut HashMap<&str, Value>)
{
    map.insert("groups", Value::Vec(Vec::new()));
    map.insert("days", Value::Int(5));
    
    let matches = App::new("VK group parser")
        .version("0.1")
        .author("KrutNA <krutna@pm.me>")
        .about("Parsing VK groups and finds smth with required texts.")
        .arg(Arg::with_name("config")
             .short("c").long("config")
             .help("Sets custom configuration file [default: config.yml]")
             .takes_value(true)
             .value_name("FILE")
             .validator(|name| {
                 match File::open(&name) {
                     Ok(_) => Ok(()),
                     Err(_) => Err(format!("Configuration file \"{}\" not found", name)),
                 }
             }))
        .arg(Arg::with_name("token")
             .short("t").long("token")
             .help("Sets token")
             .takes_value(true)
             .value_name("TOKEN"))
        .arg(Arg::with_name("groups")
             .short("g").long("groups")
             .help("Adds custom group ids and search patterns")
             .takes_value(true)
             .multiple(true)
             .value_name("ID:PATTERN")
             .validator(|val| {
                 match Regex::new(r"^[+-]?\d+:.+$").unwrap().is_match(&val) {
                     true => match val.split(":").collect::<Vec<&str>>()[0].parse::<i64>() {
                         Ok(_) => Ok(()),
                         Err(v) => Err(format!("Can't parse: \"{}\" as i64", v)),
                     },
                     false => Err(format!("Can't parse: \"{}\" as ID:PATTERN", val)),
                 }
             }))
        .arg(Arg::with_name("days")
             .short("d").long("days")
             .help("Sets custom days count [default: 5]")
             .takes_value(true)
             .value_name("COUNT")
             .validator(|val| {
                 match val.parse::<u64>() {
                     Ok(_) => Ok(()),
                     Err(_) => panic!("Can't parse: \"{}\" as u64", val),
                 }
             }))
        .get_matches();
    let config = matches.value_of("config").unwrap_or("config.yml");
    match File::open(config) {
        Ok(mut f) => {
            let mut v = String::new();
            f.read_to_string(&mut v).unwrap();
            let doc = match YamlLoader::load_from_str(&v) {
                Ok(v) => v,
                Err(_) => panic!("Can't load config \"{}\"", config),
            };
            if doc[0]["groups"].is_array() {
                for val in doc[0]["groups"].as_vec().unwrap() { match &val {
                    Yaml::String(v) if Regex::new(r"^[+-]?\d+:.+$").unwrap().is_match(&v) => {
                        let vals = v.split(":").collect::<Vec<&str>>();
                        match vals[0].parse::<i64>() {
                            Ok(v) => map.get_mut("groups").unwrap()
                                .as_mut_vec().unwrap().push(SearchArg {
                                    group_id: v,
                                    pattern: String::from(vals[1]),
                                }),
                            _ => {},
                        }
                    },
                    _ => {},
                }}
            };
            match doc[0]["days"] {
                Yaml::Integer(_) => {
                    map.insert("days", Value::Int(
                        match u64::try_from(doc[0]["days"].as_i64().unwrap()) {
                            Ok(v) => v,
                            _ => panic!(format!("Can't get u64 from \"{}\"",
                                                doc[0]["days"].as_i64().unwrap()))
                        }
                    ));
                },
                _ => {},
            }
            match doc[0]["token"] {
                Yaml::String(ref s) => {
                    map.insert("token", Value::Str(s.clone()));
                },
                _ => {},
            }
        },
        Err(_) => {},
    }
    if matches.is_present("token") || map.contains_key("token") {
        if !map.contains_key("token") {
            map.insert("token", Value::Str(String::from(matches.value_of("token").unwrap())));
        }
    } else {
        panic!("Can't find token in config or in arguments");
    }
    if matches.is_present("days") {
        map.insert("days", Value::Int(matches.value_of("days").unwrap().parse().unwrap()));
    }
    if matches.is_present("groups") {
        for val in matches.values_of("groups").unwrap() {
            let gr = val.split(":").collect::<Vec<&str>>();
            map.get_mut("groups").unwrap().as_mut_vec().unwrap().push(SearchArg {
                group_id: gr[0].parse::<i64>().unwrap(),
                pattern: String::from(gr[1]),
            });
        };
    };
}

