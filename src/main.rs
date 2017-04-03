#[macro_use] extern crate nickel;
use nickel::{Nickel, HttpRouter, StaticFilesHandler, Mountable, MediaType, NickelError, Action, Halt, Request};

fn main() {
    let mut srv = Nickel::new();

    // Serve contents build by "npm run build"
    srv.utilize(middleware! {|request|
        println!("request: {:?}", request.origin.uri);
    });
    srv.mount("/", StaticFilesHandler::new("./dist"));
    srv.get("/data/", middleware! {|request, mut response|
        response.set(MediaType::Json);
        r#"[{
    "name": "太郎",
    "yesterday": [{
        "title": "チケット#2"
    },
    {
        "title": "チケット#4"
    }],
    "today": [{"title": "チケット#3"}],
    "problem": [{"title": "結合テスト環境が動いていない"}]
}]"#
    });

    fn custom_404<'a>(err: &mut NickelError, _req: &mut Request) -> Action {
        println!("error: {:?} {:?}", err.message, _req.origin.headers);
        Halt(())
    }
    let custom_handler: fn(&mut NickelError, &mut Request) -> Action = custom_404;
    srv.handle_error(custom_handler);

    srv.listen("127.0.0.1:8000").expect("Failed to launch server");
}
