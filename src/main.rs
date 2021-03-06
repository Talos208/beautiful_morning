#[macro_use]
extern crate nickel;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate chrono;
#[macro_use]
extern crate hyper;
extern crate sha2;
extern crate base64;

use nickel::{Nickel, HttpRouter, StaticFilesHandler, Mountable, MediaType, JsonBody, Request,
             Response, MiddlewareResult};
use nickel::status::StatusCode;
use nickel::extensions::Redirect;
use hyper::header::{Authorization, Bearer};
use base64::{encode, decode};
use std::collections::HashMap;
use std::sync::RwLock;
use std::cmp::{Eq, PartialEq};
use std::io::Read;
use std::borrow::BorrowMut;
use std::str::FromStr;
use chrono::prelude::*;
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct Member {
    pub name: String,
    pub account: String,
    pub password: String, // FIXME encrypt!
}

impl PartialEq for Member {
    fn eq(&self, other: &Member) -> bool {
        self.account == other.account
    }
}
impl Eq for Member {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Work {
    pub title: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Issue {
    pub title: String,
}

#[derive(Serialize)]
pub struct JsonEntry<'a> {
    pub member: &'a String,
    pub done: Vec<Work>,
    pub to_do: Vec<Work>,
    pub problem: Vec<Issue>,
}

#[derive(Serialize)]
pub struct JsonDaily<'a> {
    pub date: String,
    pub entries: Vec<JsonEntry<'a>>,
}

#[derive(Serialize)]
pub struct Entry {
    pub done: Vec<Work>,
    pub to_do: Vec<Work>,
    pub problem: Vec<Issue>,
}

pub struct Sprint {
    pub members: Vec<Member>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AuthRequest {
    account: String,
    password: String,
}

#[derive(Serialize, Debug)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub explain: String,
}

pub struct ServerData {
    pub accounts: Vec<Member>,
    pub auth: HashMap<String, Member>,
    pub daily: HashMap<Member, Entry>,
}

static AUTH_SECRET: &'static str = "some_secret_key";

impl ServerData {
    pub fn new() -> ServerData {
        ServerData {
            accounts: vec![],
            auth: HashMap::new(),
            daily: HashMap::new(),
        }
    }

    pub fn seed() -> ServerData {
        let mut sv_data = ServerData {
            accounts: vec![],
            auth: HashMap::new(),
            daily: HashMap::new(),
        };
        let taro = Member {
            name: "太郎".to_string(),
            account: "taro".to_string(),
            password: "secret".to_string(),
        };

        sv_data.add_member(taro);
        sv_data.daily.insert(sv_data.accounts.last().unwrap().clone(), Entry {
            done: vec![Work { title: "チケット#2".to_string() }, Work { title: "チケット#4".to_string() }],
            to_do: vec![Work { title: "チケット#3".to_string() }],
            problem: vec![Issue { title: "結合テスト環境が動いていない".to_string() }]
        });

        let hanako = Member {
            name: "花子".to_string(),
            account: "hana".to_string(),
            password: "abc123".to_string(),
        };
        sv_data.add_member(hanako);
        sv_data.daily.insert(sv_data.accounts.last().unwrap().clone(), Entry {
            done: vec![Work { title: "チケット#1".to_string() }],
            to_do: vec![Work { title: "チケット#1".to_string() }],
            problem: vec![Issue { title: "チケット＃１が終わらなくて大変".to_string() }]
        });
        sv_data
    }

    pub fn is_member(&self, acc: &String) -> bool {
        for m in self.accounts.iter() {
            if *acc == m.account {
                return true;
            }
        }
        false
    }

    pub fn is_authorized(&self, acc: &String, pwd: &String) -> bool {
        for m in self.accounts.iter() {
            if *acc == m.account && *pwd == m.password {
                return true;
            }
        }
        false
    }

    pub fn authorize(&mut self, acc: &String, pwd: &String) -> String {
        let mut crypt = Sha256::default();
        crypt.input(AUTH_SECRET.as_bytes());
        crypt.input(format!("{:b}", UTC::now().timestamp()).as_bytes());
        crypt.input(acc.as_bytes());
        crypt.input(pwd.as_bytes());

        let js = base64::encode(crypt.result().as_slice());
        let m = self.member_called(acc).unwrap();
        self.auth.insert(js.clone(), m);
        js
    }

    pub fn add_member(&mut self, m: Member) -> &ServerData {
        self.accounts.push(m);
        return self;
    }

    pub fn member_called(&self, acc: &String) -> Option<Member> {
        for m in self.accounts.iter() {
            if m.account == *acc {
                return Some(m.clone()); // FIXME WeakRefとかを返す
            }
        }
        None
    }

    pub fn to_json_entry(&self) -> JsonDaily {
        let mut result = JsonDaily {
            date: "".to_string(),
            entries: vec![],
        };

        for m in self.accounts.iter() {
            match self.daily.get(m) {
                Some(v) => {
                    result.entries.push(JsonEntry {
                                            member: &m.name,
                                            done: v.done.clone(),
                                            to_do: v.to_do.clone(),
                                            problem: v.problem.clone(),
                                        })
                }
                None => {
                    result.entries.push(JsonEntry {
                                            member: &m.name,
                                            done: vec![],
                                            to_do: vec![],
                                            problem: vec![],
                                        })
                }
            };
        }
        result.date = Local::today().format("%F").to_string();
        result
    }

    pub fn entries_for(&self, m: &Member) -> Entry {
        let mut result = Entry {
            done: vec![],
            to_do: vec![],
            problem: vec![],
        };

        match self.daily.get(m) {
            Some(v) => {
                result.done = v.done.clone();
                result.to_do = v.to_do.clone();
                result.problem = v.problem.clone();
            }
            None => {}
        }

        return result;
    }

    pub fn update_done(&mut self, user: &Member, done: &Vec<Work>) -> bool {
        match self.daily.get_mut(user) {
            Some(v) => {
                v.done = done.to_vec();
                true
            }
            None => false,
        }
    }

    pub fn delete_done_at(&mut self, user: &Member, id: usize) -> bool {
        match self.daily.get_mut(user) {
            Some(v) => {
                v.done.remove(id);
                true
            }
            None => false,
        }
    }

    pub fn update_todo(&mut self, user: &Member, todo: &Vec<Work>) -> bool {
        match self.daily.get_mut(user) {
            Some(v) => {
                v.to_do = todo.to_vec();
                true
            }
            None => false,
        }
    }

    pub fn delete_todo_at(&mut self, user: &Member, id: usize) -> bool {
        match self.daily.get_mut(user) {
            Some(v) => {
                v.to_do.remove(id);
                true
            }
            None => false,
        }
    }

    pub fn update_problem(&mut self, user: &Member, problem: &Vec<Issue>) -> bool {
        match self.daily.get_mut(user) {
            Some(v) => {
                v.problem = problem.to_vec();
                true
            }
            None => false,
        }
    }

    pub fn delete_problem_at(&mut self, user: &Member, id: usize) -> bool {
        match self.daily.get_mut(user) {
            Some(v) => {
                v.problem.remove(id);
                true
            }
            None => false,
        }
    }
}

header! { (XRequestUser, "X-Request-User") => [String] }

fn authenticator<'mw, 'conn, 'd>(request: &mut Request<'mw, 'conn, RwLock<ServerData>>,
                                 response: Response<'mw, RwLock<ServerData>>)
                                 -> MiddlewareResult<'mw, RwLock<ServerData>> {
    let uac: String;
    {
        let auth = request.origin.headers.get::<Authorization<Bearer>>();
        match auth {
            Some(a) => {
                let data = request.server_data().read().unwrap();
                let au = data.auth.get(&a.token);
                match au {
                    Some(u) => uac = u.account.clone(),
                    None => return response.error(StatusCode::Forbidden, "Access denied"),
                }
            }
            None => return response.error(StatusCode::Forbidden, "Access denied"),
        }
    }
    request.origin.headers.set(XRequestUser(uac));
    return response.next_middleware();
}

fn main() {
    let server_data = RwLock::new(ServerData::seed());

    {
        let mut srv = Nickel::with_data(server_data);

        srv.utilize(middleware! {|request|
            println!("request: {:?}", request.origin.uri);
        });
        srv.mount("/", StaticFilesHandler::new("./dist"));
        srv.post("/login/",
                 middleware! {|request, mut response| < RwLock<ServerData> >
            let mut reqbuf: String = String::new();
            request.origin.read_to_string(reqbuf.borrow_mut());
            let auth: AuthRequest =
                match serde_json::from_str(reqbuf.as_str()) {
                    Ok(x) => x,
                    Err(e) => return response.error(StatusCode::BadRequest, format!("{:?}", e))
                };
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
            match serde_json::to_string(&res) {
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
        srv.post("/register/",
                 middleware! {|request, mut response| < RwLock<ServerData> >
            let mut reqbuf: String = String::new();
            request.origin.read_to_string(reqbuf.borrow_mut());
            let reg0 = serde_json::from_str(reqbuf.as_str());
            let reg: Member =
            match reg0 {
                Ok(x) => x,
                Err(e) => return response.error(StatusCode::BadRequest, format!("{:?}", e))
            };
            response.set(MediaType::Json);
            if response.data().read().unwrap().is_member(&reg.account) {
                response.set(StatusCode::BadRequest);
                return response.send("{\"success\": \"false\"}")
            }
            response.server_data().write().unwrap().add_member(reg);

            response.set(StatusCode::TemporaryRedirect);
            return response.redirect("/")
        });

        // これ以降は認証が必要
        srv.utilize(authenticator);
        srv.get("/data/",
                middleware! {|request, mut response| < RwLock<ServerData> >
            let dt = &response.data().read().unwrap();
            let result = dt.to_json_entry();
            match serde_json::to_string(&result) {
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
        srv.get("/entry/:date",
                middleware! {|request, mut response| < RwLock<ServerData> >
            let ref hdr = request.origin.headers;
            let ref run = hdr.get::<XRequestUser>().unwrap().0;
            println!("Request user for entry is {:?}", run);

            let date = request.param("date");
            match date {
                Some(ref d) => {
                    println!("{:?}", d);
                },
                None => {
                    return response.error(StatusCode::NotFound, "Fail to get date")
                }
            };

            let dt = &response.data().read().unwrap();
            let ru = dt.member_called(run).unwrap();
            let ent = dt.entries_for(&ru);
            match serde_json::to_string(&ent) {
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
        srv.post("/entry/:date/done",
                 middleware! {|request, mut response| < RwLock<ServerData> >
            let mut reqbuf: String = String::new();
            request.origin.read_to_string(reqbuf.borrow_mut());
            let req: Result<Vec<Work>, _> = serde_json::from_str(reqbuf.as_str());
            match req {
                Ok(x) => {
//                    println!("Done {:?}", x);
                    let ref run = request.origin.headers.get::<XRequestUser>().unwrap().0;
                    let ref mut dt = response.data().write().unwrap();
                    let ref ru = dt.member_called(run).unwrap();
                    dt.update_done(ru, &x);
                },
                Err(e) => return response.error(StatusCode::InternalServerError, format!("{:?}", e))
            };

            response.set(MediaType::Json);
            "{}"
        });
        srv.delete("/entry/:date/done/:id",
                   middleware! {|request, mut response| < RwLock<ServerData> >
            let id_param = request.param("id");
            let id: usize = match id_param {
                Some(ref d) => {
                    match d.parse::<usize>() {
                      Ok(v) => v,
                      Err(e) => {
                        return response.error(StatusCode::BadRequest, format!("{:?}", e))
                      }
                    }
                },
                None => {
                    return response.error(StatusCode::NotFound, "Fail to get date")
                }
            };

            let ref run = request.origin.headers.get::<XRequestUser>().unwrap().0;
            let ref mut dt = response.data().write().unwrap();
            let ref ru = dt.member_called(run).unwrap();
            dt.delete_done_at(ru, id);

            response.set(MediaType::Json);
            "{}"
        });
        srv.post("/entry/:date/todo",
                 middleware! {|request, mut response| < RwLock<ServerData> >
            let mut reqbuf: String = String::new();
            request.origin.read_to_string(reqbuf.borrow_mut());
            let req: Result<Vec<Work>,_> = serde_json::from_str(reqbuf.as_str());
            match req {
                Ok(x) => {
//                    println!("Todo {:?}", x)
                    let ref run = request.origin.headers.get::<XRequestUser>().unwrap().0;
                    let ref mut dt = response.data().write().unwrap();
                    let ref ru = dt.member_called(run).unwrap();
                    dt.update_todo(ru, &x);
                },
                Err(e) => return response.error(StatusCode::InternalServerError, format!("{:?}", e))
            };

            response.set(MediaType::Json);
            "{}"
        });
        srv.delete("/entry/:date/todo/:id",
                   middleware! {|request, mut response| < RwLock<ServerData> >
            let id_param = request.param("id");
            let id: usize = match id_param {
                Some(ref d) => {
                    match d.parse::<usize>() {
                      Ok(v) => v,
                      Err(e) => {
                        return response.error(StatusCode::BadRequest, format!("{:?}", e))
                      }
                    }
                },
                None => {
                    return response.error(StatusCode::NotFound, "Fail to get date")
                }
            };

            let ref run = request.origin.headers.get::<XRequestUser>().unwrap().0;
            let ref mut dt = response.data().write().unwrap();
            let ref ru = dt.member_called(run).unwrap();
            dt.delete_todo_at(ru, id);

            response.set(MediaType::Json);
            "{}"
        });
        srv.post("/entry/:date/problem",
                 middleware! {|request, mut response| < RwLock<ServerData> >
            let mut reqbuf: String = String::new();
            request.origin.read_to_string(reqbuf.borrow_mut());
            let req: Result<Vec<Issue>,_> = serde_json::from_str(reqbuf.as_str());
            match req {
                Ok(x) => {
//                    println!("Problem {:?}", x)
                    let ref run = request.origin.headers.get::<XRequestUser>().unwrap().0;
                    let ref mut dt = response.data().write().unwrap();
                    let ref ru = dt.member_called(run).unwrap();
                    dt.update_problem(ru, &x);
                },
                Err(e) => return response.error(StatusCode::InternalServerError, format!("{:?}", e))
            };

            response.set(MediaType::Json);
            "{}"
        });
        srv.delete("/entry/:date/problem/:id",
                   middleware! {|request, mut response| < RwLock<ServerData> >
            let id_param = request.param("id");
            let id: usize = match id_param {
                Some(ref d) => {
                    match d.parse::<usize>() {
                      Ok(v) => v,
                      Err(e) => {
                        return response.error(StatusCode::BadRequest, format!("{:?}", e))
                      }
                    }
                },
                None => {
                    return response.error(StatusCode::NotFound, "Fail to get date")
                }
            };

            let ref run = request.origin.headers.get::<XRequestUser>().unwrap().0;
            let ref mut dt = response.data().write().unwrap();
            let ref ru = dt.member_called(run).unwrap();
            dt.delete_problem_at(ru, id);

            response.set(MediaType::Json);
            "{}"
        });

        srv.listen("127.0.0.1:8000").expect("Failed to launch server");
    }
}
