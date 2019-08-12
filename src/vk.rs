use crate::init::{Convert};
use crate::init as init;
use std::collections::HashMap;
use try_from::TryFrom;
use std::time::SystemTime;
use serde_json::Value;
use std::fs;

struct Param
{
    id: i64,
    pattern: String,
    before_time: u64,
    token: String,
}

fn parse(
    param: Param)
{
    const METHOD_URL: &str = "https://api.vk.com/method/wall.search";
    const COUNT: u64 = 100;
    let mut vec: Vec<String> = Vec::new();
    let mut is_break = false;
    let mut offset = 0u64; 
    loop {
        if is_break { break; }
        let res: Value = reqwest::get(
            &format!("{}?v=5.101&access_token={}&owner_id={}&query={}&count={}&offset={}",
                     METHOD_URL, param.token, param.id, param.pattern, COUNT, offset))
            .expect("Can't access to api.")
            .json().unwrap();
        if res.as_object().unwrap().contains_key("error") {
            panic!(format!("Got error: \"{}\". Request params: [{}]",
                           res.as_object().unwrap().get("error").unwrap()
                           .as_object().unwrap().get("error_msg").unwrap()
                           .as_str().unwrap(),
                           res.as_object().unwrap().get("error").unwrap()
                           .as_object().unwrap().get("request_params").unwrap()
                           .as_array().unwrap().iter().map(|param| {
                               format!("{}:{}",
                                       param.as_object().unwrap().get("key").unwrap()
                                       .as_str().unwrap(),
                                       param.as_object().unwrap().get("value").unwrap()
                                       .as_str().unwrap()
                               )
                           }).collect::<Vec<String>>().join(", ")));
        } else {
            for res in res.as_object().unwrap().get("response").unwrap()
                .as_object().unwrap().get("items").unwrap()
                .as_array().unwrap() {
                    if u64::try_from(res.as_object().unwrap()
                                     .get("date").unwrap().as_i64().unwrap())
                        .unwrap() < param.before_time {
                            is_break = true;
                            break;
                        }
                    vec.push(format!("post_id: {}",
                                     match res.as_object().unwrap()
                                     .get("post_type").unwrap().as_str().unwrap() {
                                         "reply" => res.as_object().unwrap()
                                             .get("post_id").unwrap().as_i64().unwrap(),
                                         _ => res.as_object().unwrap()
                                             .get("id").unwrap().as_i64().unwrap()}));
                    vec.push(res.as_object().unwrap().get("text").unwrap()
                             .as_str().unwrap().split("\n")
                             .collect::<Vec<&str>>().iter()
                             .map(|v| format!("    | {}", v))
                             .collect::<Vec<String>>().join("\n"))
                }
        }
        if res.as_object().unwrap().get("response").unwrap()
            .as_object().unwrap().get("count").unwrap()
            .as_u64().unwrap() < COUNT { break; }
        offset += COUNT;
    }
    fs::create_dir_all(format!("result-{}/", param.before_time)).unwrap();
    fs::write(format!("result-{}/{}",
                      param.before_time, param.id),
              vec.join("\n---\n")).expect("Can't write to file");
} 

pub fn search(
    map: &HashMap<&str, init::Value>)
{
    let before_time =
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() -
        map.get("days").unwrap().as_u64().unwrap() * 60 * 60 * 24;
    let handlers: Vec<_> = map.get("groups").unwrap()
        .as_vec().unwrap().iter().map(|search_arg| {
        let s_arg = Param{ id: search_arg.group_id,
                           pattern: search_arg.pattern.clone(),
                           before_time,
                           token: map.get("token").unwrap().as_str().unwrap(), };
        std::thread::spawn(move || parse(s_arg))})
        .collect();
    for h in handlers {
        match h.join() {
            Ok(_) => (),
            Err(v) => panic!(v),
        }
    }
}
