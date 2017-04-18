#[macro_use] extern crate nickel;
extern crate rustc_serialize;
extern crate chrono;
extern crate crypto;
#[macro_use] extern crate hyper;

use nickel::{Nickel, HttpRouter, StaticFilesHandler, Mountable, MediaType, JsonBody, Request, Response, MiddlewareResult};
use nickel::status::StatusCode;
use nickel::extensions::Redirect;
use hyper::header::{ Authorization, Bearer };
use rustc_serialize::{json, base64};
use rustc_serialize::base64::ToBase64;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::cmp::{Eq, PartialEq};
//use std::str::FromStr;
//use std::default::Default;
use chrono::prelude::*;
use crypto::sha3::Sha3;
use crypto::digest::Digest;

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

#[derive(RustcEncodable, Debug)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub explain: String
}

pub struct ServerData {
    pub accounts: Vec<Member>,
    pub auth: HashMap<String, String>
}

static AUTH_SECRET: &'static str = "some_secret_key";

impl ServerData {
    pub fn new() -> ServerData {
        ServerData { accounts: vec![], auth: HashMap::new() }
    }

    pub fn is_member(&self, acc: &String) -> bool {
        for m in self.accounts.iter() {
            if *acc == m.account {
                return true
            }
        }
        false
    }

    pub fn is_authorized(&self, acc: &String, pwd: &String) -> bool {
        for m in self.accounts.iter() {
            if *acc == m.account && *pwd == m.password {
                return true
            }
        }
        false
    }

    pub fn authorize(&mut self, acc: &String, pwd: &String) -> String {
        let mut crypt = Sha3::sha3_224();
        crypt.reset();
        crypt.input_str(format!("{:b}", UTC::now().timestamp()).as_str());
        crypt.input_str(acc);
        crypt.input_str(pwd);
        crypt.input_str(AUTH_SECRET);
        let result = crypt.result_str();

        let js = result.as_bytes().to_base64(base64::Config{
            char_set: base64::CharacterSet::Standard,
            newline: base64::Newline::CRLF,
            pad: false,
            line_length: None
        });
        self.auth.insert(js.clone(), acc.clone());
        js
    }

    pub fn add(&mut self, m: Member) -> &ServerData {
        self.accounts.push(m);
        return self
    }
}

header! { (XRequestUser, "X-Request-User") => [String] }

fn authenticator<'mw, 'conn>(request: &mut Request<'mw, 'conn,  Arc<RwLock<ServerData>>>, response: Response<'mw,  Arc<RwLock<ServerData>>>) ->MiddlewareResult<'mw, Arc<RwLock<ServerData>>> {
    let uac: String;
    {
        let auth = request.origin.headers.get::<Authorization<Bearer>>();
        match auth {
            Some(a) => {
                let data = request.server_data().read().unwrap();
                let au = data.auth.get(&a.token);
                match au {
                    Some(u) => {
                        uac = u.clone()
                    },
                    None => return response.error(StatusCode::Forbidden, "Access denied")
                }
            },
            None => return response.error(StatusCode::Forbidden, "Access denied")
        }
    }
    request.origin.headers.set(XRequestUser(uac));
    return response.next_middleware()
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
        srv.post("/login/", middleware! {|request, mut response| < Arc<RwLock<ServerData>> >
            let auth = request.json_as::<AuthRequest>().ok().unwrap();
            let acc = &auth.account;
            let pwd = &auth.password;
            let res =
                if response.data().read().unwrap().is_authorized(acc, pwd) {
                    let jwt = response.data().write().unwrap().authorize(acc, pwd);
                    AuthResponse{
                        success: true,
                        token: Some(jwt),
                        explain: String::from("Login succeed.")
                    }
                } else {
                    response.set(StatusCode::Forbidden);
                    AuthResponse{
                        success: false,
                        token: None,
                        explain: String::from("Login fail.")
                    }
                };
            match json::encode(&res) {
                Ok(js) => {
                    response.set(MediaType::Json);
                    js
                },
                Err(e) => {
                    response.set(StatusCode::InternalServerError);
                    format!("{:?}", e)
                }
            }
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

        // これ以降は認証が必要
        srv.utilize(authenticator);
        srv.get("/data/", middleware! {|request, mut response| < Arc<RwLock<ServerData>> >
            {
                let ref hdr = request.origin.headers;
                let ref ru = hdr.get::<XRequestUser>().unwrap().0;
                println!("Request user is {:?}", ru);
            }
            let dt = &response.data().read().unwrap();
            let result = convert_to_json_entry(dt, &daily);
            match json::encode(&result) {
                Ok(js) => {
                    response.set(MediaType::Json);
                    js
                },
                Err(e) => {
                    response.set(StatusCode::InternalServerError);
                    format!("{:?}", e)
                }
            }
        });

        srv.listen("127.0.0.1:8000").expect("Failed to launch server");
    }
}
