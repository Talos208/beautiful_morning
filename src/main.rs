#[macro_use]
extern crate nickel;
extern crate rustc_serialize;

use nickel::{Nickel, HttpRouter, StaticFilesHandler, Mountable, MediaType, NickelError, Action, Halt, Request };
use rustc_serialize::json;

#[derive(RustcEncodable)]
pub struct Member {
    pub name: String,
}

#[derive(RustcEncodable)]
pub struct Work {
    pub title: String,
}

#[derive(RustcEncodable)]
pub struct Issue {
    pub title: String,
}

#[derive(RustcEncodable)]
pub struct Entry {
    pub member: Member,
    pub yesterday: Vec<Work>,
    pub today: Vec<Work>,
    pub problem: Vec<Issue>
}

fn main() {
    let daily = vec![Entry {
        member: Member { name: "太郎".to_string() },
        yesterday: vec![Work { title: "チケット#2".to_string() }, Work { title: "チケット#4".to_string() }],
        today: vec![Work { title: "チケット#3".to_string() }],
        problem: vec![Issue { title: "結合テスト環境が動いていない".to_string() }]
    }];
    let mut srv = Nickel::new();

    // Serve contents build by "npm run build"
    srv.utilize(middleware! {|request|
        println!("request: {:?}", request.origin.uri);
    });
    srv.mount("/", StaticFilesHandler::new("./dist"));
    srv.get("/data/", middleware! {|_, mut response|
        response.set(MediaType::Json);
        json::encode(&daily).unwrap()
    });

    fn custom_404<'a>(err: &mut NickelError, _req: &mut Request) -> Action {
        println!("error: {:?} {:?}", err.message, _req.origin.headers);
        Halt(())
    }
    let custom_handler: fn(&mut NickelError, &mut Request) -> Action = custom_404;
    srv.handle_error(custom_handler);

    srv.listen("127.0.0.1:8000").expect("Failed to launch server");
}
