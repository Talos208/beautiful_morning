#[macro_use] extern crate nickel;
use nickel::{Nickel, StaticFilesHandler, Mountable};

fn main() {
    let mut srv = Nickel::new();

    // Serve contents build by "npm run build" 
    srv.mount("/", StaticFilesHandler::new("./dist"));

    srv.listen("127.0.0.1:8000").expect("Failed to launch server");
}
