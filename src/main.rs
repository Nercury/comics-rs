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
use std::convert::AsRef;

use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use std::path::Path;
use std::sync::Mutex;
use std::sync::Arc;
use mount::Mount;
use staticfile::Static;
use router::Router;

fn append_link(
    vals: &mut globals::Globals,
    disabled_key: &'static str,
    href_key: &'static str,
    href: Option<String>
) {
    vals.amend(disabled_key, match href {
        None => "disabled".into(),
        _ => "".into(),
    });
    vals.amend(href_key, match href {
        Some(href) => href,
        None => "javascript:;".into(),
    });
}

fn send_page(index: &mut index::Index, req: &mut Request) -> IronResult<Response> {
    match req.extensions.get::<Router>()
        .unwrap().find("slug") {
            Some(ref slug) => {
                match index.find(slug) {
                    Some(found) => {
                        let mut vals = globals::Globals::new()
                            .with("title", found.title)
                            .with("file", ["/i/", found.file.to_string_lossy().as_ref()].concat())
                            .with("width", "".into())
                            .with("height", "".into());

                        append_link(&mut vals, "first_disabled", "first_href", match index.first_slug() {
                            Some(slug) => if slug == found.slug {
                                None
                            } else {
                                Some(["/c/", slug.as_ref()].concat())
                            },
                            None => None,
                        });

                        append_link(&mut vals, "prev_disabled", "prev_href", match found.prev_slug {
                            Some(slug) => Some(["/c/", slug.as_ref()].concat()),
                            None => None,
                        });

                        append_link(&mut vals, "random_disabled", "random_href", None);

                        append_link(&mut vals, "next_disabled", "next_href", match found.next_slug {
                            Some(slug) => Some(["/c/", slug.as_ref()].concat()),
                            None => None,
                        });

                        append_link(&mut vals, "last_disabled", "last_href", match index.last_slug() {
                            Some(slug) => if slug == found.slug {
                                None
                            } else {
                                Some(["/c/", slug.as_ref()].concat())
                            },
                            None => None,
                        });

                        let parsed = template::parse(
                            "static/plain/comic.html",
                            vals
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
        .mount("/c/", router)
        .mount("/css/", Static::new(Path::new("public/css")))
        .mount("/js/", Static::new(Path::new("public/js")))
        .mount("/font/", Static::new(Path::new("public/font")))
        .mount("/i/", Static::new(Path::new("data/images")))
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
