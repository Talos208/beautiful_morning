#[macro_use]
extern crate nickel;
extern crate rustc_serialize;
extern crate chrono;

use nickel::{Nickel, HttpRouter, StaticFilesHandler, Mountable, MediaType, JsonBody};
use nickel::status::StatusCode;
use nickel::extensions::Redirect;
use rustc_serialize::json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::cmp::{Eq, PartialEq};
use chrono::prelude::*;

#[derive(RustcEncodable, RustcDecodable, Hash)]
pub struct Member {
    pub name: String,
    pub account: String,
    pub password: String,   // FIXME encrypt!
}

impl PartialEq for Member {
    fn eq(&self, other: &Member) -> bool {
        self.account == other.account
    }
}
impl Eq for Member {
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
    pub member: &'a String,
    pub done: Vec<Work>,
    pub to_do: Vec<Work>,
    pub problem: Vec<Issue>
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

fn convert_to_json_entry<'a>(acc: &'a ServerData, map: &'a HashMap<String, Box<Entry>>) -> JsonDaily<'a> {
    let mut result = JsonDaily{date: "".to_string(), entries:vec![]};

    for m in acc.accounts.iter() {
        match map.get(&m.name) {
            Some(v) => result.entries.push(JsonEntry{
                member: &m.name,
                done: v.done.clone(),
                to_do: v.to_do.clone(),
                problem: v.problem.clone()
            }),
            None => result.entries.push(JsonEntry{
                member: &m.name,
                done: vec![],
                to_do: vec![],
                problem: vec![]
            }),
        };
    }
    result.date = Local::today().format("%F").to_string();
    result
}

#[derive(RustcDecodable, Clone, Debug)]
pub struct AuthRequest {
    account: String,
    password: String
}


//fn authenticator<'mw>(request: &mut Request, response: Response<'mw>, ) ->MiddlewareResult<'mw> {
//    response.redirect("/login")
////    response.error(StatusCode::Forbidden, "Access denied")
//}

pub struct ServerData {
    pub accounts: Vec<Member>
}

impl ServerData {
    pub fn new() -> ServerData {
        ServerData { accounts: vec![] }
    }

    pub fn is_member(&self, acc: &String) -> bool {
        for m in self.accounts.iter() {
            if *acc == m.account {
                return true
            }
        }
        false
    }

    pub fn authorize(&self, acc: &String, pwd: String) -> bool {
        for m in self.accounts.iter() {
            if *acc == m.account && pwd == m.password {
                return true
            }
        }
        false
    }

    pub fn add(&mut self, m: Member) -> &ServerData {
        self.accounts.push(m);
        return self
    }
}


fn main() {
    let taro = Member { name: "太郎".to_string(), account: "taro".to_string(), password: "secret".to_string() };
    let hanako = Member { name: "花子".to_string(), account: "hana".to_string(), password: "abc123".to_string() };

    let mut daily: HashMap<String, Box<Entry>> = HashMap::new();

    daily.insert(taro.name.clone(), Box::new(Entry {
        done: vec![Work { title: "チケット#2".to_string() }, Work { title: "チケット#4".to_string() }],
        to_do: vec![Work { title: "チケット#3".to_string() }],
        problem: vec![Issue { title: "結合テスト環境が動いていない".to_string() }]
    }));
    daily.insert(hanako.name.clone(), Box::new(Entry {
        done: vec![Work { title: "チケット#1".to_string() }],
        to_do: vec![Work { title: "チケット#1".to_string() }],
        problem: vec![Issue { title: "チケット＃１が終わらなくて大変".to_string() }]
    }));
    let mut sv_data = ServerData::new();
    sv_data.add(taro);
    sv_data.add(hanako);

    let server_data = Arc::new(RwLock::new(sv_data));

    {
        let mut srv = Nickel::with_data(server_data);

        srv.utilize(middleware! {|request|
            println!("request: {:?}", request.origin.uri);
        });
        srv.mount("/", StaticFilesHandler::new("./dist"));
        srv.get("/data/", middleware! {|_, mut response| < Arc<RwLock<ServerData>> >
            response.set(MediaType::Json);
            let dt = &response.data().read().unwrap();
            let result = convert_to_json_entry(dt, &daily);
            json::encode(&result).unwrap()
        });
        srv.post("/login/", middleware! {|request, mut response| < Arc<RwLock<ServerData>> >
            let auth = request.json_as::<AuthRequest>().ok().unwrap();
            let acc = &auth.account;
            let pwd = auth.password;
            if response.data().read().unwrap().authorize(acc, pwd) {
                response.set(MediaType::Json);
                return response.send("{\"success\": \"true\", \"token\": \"123456789\"}")
            }
            response.set(StatusCode::Forbidden);
            "{\"success\": \"false\"}"
        });

        srv.post("/register/", middleware! {|request, mut response| <Arc<RwLock<ServerData>> >
            let reg = request.json_as::<Member>().ok().unwrap();
            response.set(MediaType::Json);
            if response.data().read().unwrap().is_member(&reg.account) {
                response.set(StatusCode::BadRequest);
                return response.send("{\"success\": \"false\"}")
            }
            response.server_data().write().unwrap().add(reg);

            response.set(StatusCode::TemporaryRedirect);
            return response.redirect("/")
        });

        srv.listen("127.0.0.1:8000").expect("Failed to launch server");
    }
}
