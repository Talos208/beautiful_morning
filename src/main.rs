#[macro_use]
extern crate nickel;
extern crate rustc_serialize;
extern crate chrono;

use nickel::{Nickel, HttpRouter, StaticFilesHandler, Mountable, MediaType, NickelError, Action, Halt, Request};
use rustc_serialize::json;
use std::collections::HashMap;
use chrono::prelude::*;

#[derive(RustcEncodable)]
pub struct Member {
    pub name: String,
}

#[derive(RustcEncodable, Clone)]
pub struct Work {
    pub title: String,
}

#[derive(RustcEncodable, Clone)]
pub struct Issue {
    pub title: String,
}

#[derive(RustcEncodable)]
pub struct JsonEntry<'a> {
    pub member: Member,
    pub done: &'a Vec<Work>,
    pub to_do: &'a Vec<Work>,
    pub problem: &'a Vec<Issue>
}

pub struct Daily {
    pub entries: HashMap<String, Entry>
}

#[derive(RustcEncodable)]
pub struct JsonDaily<'a> {
    pub date: String,
    pub entries: Vec<JsonEntry<'a>>
}

pub struct Entry {
    pub done: Vec<Work>,
    pub to_do: Vec<Work>,
    pub problem: Vec<Issue>
}

pub struct Sprint {
    pub members: Vec<Member>
}

fn convert_to_json_entry(map: &HashMap<String, Entry>) -> JsonDaily {
    let mut result = JsonDaily{date: "".to_string(), entries:vec![]};

    for (k, v) in map.iter() {
        let e = JsonEntry{
            member: Member{name: k.to_string()},
            done: &(v.done),
            to_do: &(v.to_do),
            problem: &(v.problem)
        };
        result.entries.push(e);
    }
    result.date = Local::today().format("%F").to_string();
    result
}

fn main() {
    let taro = Member { name: "太郎".to_string() };
    let mut daily: HashMap<String, Entry> = HashMap::new();

    daily.insert(taro.name, Entry{
        done: vec![Work { title: "チケット#2".to_string() }, Work { title: "チケット#4".to_string() }],
        to_do: vec![Work { title: "チケット#3".to_string() }],
        problem: vec![Issue { title: "結合テスト環境が動いていない".to_string() }]
    });
    daily.insert("花子".to_string(), Entry{
        done: vec![Work { title: "チケット#1".to_string() }],
        to_do: vec![Work { title: "チケット#1".to_string() }],
        problem: vec![Issue { title: "チケット＃１が終わらなくて大変".to_string() }]
    });

    let mut srv = Nickel::new();

    // Serve contents build by "npm run build"
    srv.utilize(middleware! {|request|
        println!("request: {:?}", request.origin.uri);
    });
    srv.mount("/", StaticFilesHandler::new("./dist"));
    srv.get("/data/", middleware! {|_, mut response|
        response.set(MediaType::Json);
        let result = convert_to_json_entry(&daily);
        json::encode(&result).unwrap()
    });

    fn custom_404<'a>(err: &mut NickelError, _req: &mut Request) -> Action {
        println!("error: {:?} {:?}", err.message, _req.origin.headers);
        Halt(())
    }
    let custom_handler: fn(&mut NickelError, &mut Request) -> Action = custom_404;
    srv.handle_error(custom_handler);

    srv.listen("127.0.0.1:8000").expect("Failed to launch server");
}
