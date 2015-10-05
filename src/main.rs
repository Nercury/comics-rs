extern crate iron;
extern crate hyper;
extern crate time;
extern crate mount;
extern crate router;
extern crate staticfile;
extern crate unicase;
extern crate serde;
extern crate serde_json;

mod index;
mod template;
mod globals;
mod iron_ex;
mod headers;
mod release;
mod index_models;

use iron::prelude::*;
use iron::status;
use std::mem;
use std::convert::AsRef;

use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use std::path::Path;
use std::sync::Mutex;
use std::sync::Arc;
use mount::Mount;
use staticfile::Static;
use router::Router;

static mut MAIN_HTML: *mut u8 = 0 as *mut u8;

fn send_main(req: &mut Request) -> IronResult<Response> {
    if req.url.path.len() == 1 && req.url.path[0] != "" {
        return Ok(Response::with(status::NotFound));
    }

    let html: &Box<Vec<u8>> = unsafe { mem::transmute_copy(&&MAIN_HTML) };
    let slice: &[u8] = (*html).as_ref();
    let mut response = Response::with((status::Ok, slice));

    response.headers.set(
        ContentType(
            Mime(TopLevel::Text, SubLevel::Html, vec![(Attr::Charset, Value::Utf8)])
        )
    );

    Ok(response)
}

fn send_page(index: &mut index::Index, req: &mut Request) -> IronResult<Response> {
    match req.extensions.get::<Router>()
        .unwrap().find("slug") {
            Some(ref slug) => {
                match index.find(slug) {
                    Some(found) => {
                        let parsed = template::parse(
                            "static/plain/comic.html",
                            globals::Globals::new()
                                .with("title", found.title)
                        );
                        let mut response = Response::with((status::Ok, parsed));
                        response.headers.set(
                            ContentType(
                                Mime(TopLevel::Text, SubLevel::Html, vec![(Attr::Charset, Value::Utf8)])
                            )
                        );
                        Ok(response)
                    },
                    None => Ok(Response::with(status::NotFound)),
                }
            },
            None => Ok(Response::with(status::NotFound)),
        }
}

fn main() {
    let index = Arc::new(Mutex::new(index::Index::from_file("data/index.json")));
    let tmp = Box::new(template::parse("static/plain/main.html", globals::Globals::new()));
    unsafe { MAIN_HTML = mem::transmute(tmp) };

    let mut router = Router::new();
    router.get("/:slug", move |req: &mut Request| -> IronResult<Response> {
        match index.lock() {
            Ok(mut index) => {
                send_page(&mut index, req)
            },
            Err(e) => {
                println!("Error locking index: {:?}", e);
                Ok(Response::with(status::NotFound))
            }
        }
    });

    let mut mount = Mount::new();
    mount
        .mount("/", send_main)
        .mount("/c/", router)
        .mount("/css/", Static::new(Path::new("public/css")))
        .mount("/js/", Static::new(Path::new("public/js")))
        .mount("/font/", Static::new(Path::new("public/font")))
    ;

    let mut chain = Chain::new(mount);

    chain = enable_browser_cache(chain);

    Iron::new(chain).http("localhost:3000").unwrap();
}

#[cfg(feature = "prod")]
fn enable_browser_cache(mut chain: Chain) -> Chain {
    chain.link_after(headers::StaticHeaders);
    chain
}

#[cfg(not(feature = "prod"))]
fn enable_browser_cache(mut chain: Chain) -> Chain {
    use iron_ex::response;
    chain.link_before(response::Time);
    chain.link_after(response::Time);
    chain
}
