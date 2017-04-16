#[macro_use]
extern crate nickel;
extern crate rustc_serialize;
extern crate chrono;
// extern crate core;

use nickel::{Nickel, HttpRouter, StaticFilesHandler, Mountable, MediaType, NickelError, Action, Halt, Request, Response, MiddlewareResult, JsonBody};
use nickel::status::StatusCode;
use nickel::extensions::Redirect;
// use nickel::extensions::response::Redirect::{redirect};
use rustc_serialize::json;
use std::collections::HashMap;
use std::hash::{Hash};
use std::sync::Arc;
use std::cmp::{Eq, PartialEq};
use chrono::prelude::*;
use std::io::Read;
// use core::ops::DerefMut;
// use core::borrow::BorrowMut;

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
    pub done: &'a Vec<Work>,
    pub to_do: &'a Vec<Work>,
    pub problem: &'a Vec<Issue>
}

//pub struct Daily {
//    pub entries: HashMap<String, Entry>
//}

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

fn convert_to_json_entry<'a>(map: &'a HashMap<String, Box<Entry>>) -> JsonDaily {
    let mut result = JsonDaily{date: "".to_string(), entries:vec![]};

    for (k, v) in map.iter() {
        let e = JsonEntry{
            member: &(k),
            done: &(v.done),
            to_do: &(v.to_do),
            problem: &(v.problem)
        };
        result.entries.push(e);
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

fn main() {
    let taro = Member { name: "太郎".to_string(), account: "taro".to_string(), password: "secret".to_string() };
    let hanako = Member { name: "花子".to_string(), account: "hana".to_string(), password: "abc123".to_string() };
    let members: Arc<Vec<Member>> = Arc::new(vec![
        taro,
        hanako
    ]);

    let mut daily: HashMap<String, Box<Entry>> = HashMap::new();

    daily.insert(members[0].name.clone(), Box::new(Entry {
        done: vec![Work { title: "チケット#2".to_string() }, Work { title: "チケット#4".to_string() }],
        to_do: vec![Work { title: "チケット#3".to_string() }],
        problem: vec![Issue { title: "結合テスト環境が動いていない".to_string() }]
    }));
    daily.insert(members[1].name.clone(), Box::new(Entry {
        done: vec![Work { title: "チケット#1".to_string() }],
        to_do: vec![Work { title: "チケット#1".to_string() }],
        problem: vec![Issue { title: "チケット＃１が終わらなくて大変".to_string() }]
    }));

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
    let members_for_login = members.clone();
    srv.post("/login/", middleware! {|request, mut response|
        let auth = request.json_as::<AuthRequest>().ok().unwrap();
//        println!("{:?}", auth);
        let acc = auth.account;
        let pwd = auth.password;
//        println!("{:?} {:?}", acc, pwd);
        response.set(MediaType::Json);
        for m in members_for_login.iter() {
            if acc == m.account && pwd == m.password {
                return response.send("{\"success\": \"true\", \"token\": \"123456789\"}")
            }
        }
        response.set(StatusCode::Forbidden);
        "{\"success\": \"false\"}"
    });
    let members_for_register = members.clone();
    srv.post("/register/", middleware! {|request, mut response|
        let reg = request.json_as::<Member>().ok().unwrap();
        response.set(MediaType::Json);
        for m in members_for_register.iter() {
            if reg.account == m.account {
                response.set(StatusCode::BadRequest);
                return response.send("{\"success\": \"false\"}")
            }
        }
        // TODO Create new account

        response.set(StatusCode::TemporaryRedirect);
        return response.redirect("/")
    });

    fn custom_404<'a>(err: &mut NickelError, _req: &mut Request) -> Action {
        println!("error: {:?} {:?}", err.message, _req.origin.headers);
        Halt(())
    }
    let custom_handler: fn(&mut NickelError, &mut Request) -> Action = custom_404;
    srv.handle_error(custom_handler);

    srv.listen("127.0.0.1:8000").expect("Failed to launch server");
}
