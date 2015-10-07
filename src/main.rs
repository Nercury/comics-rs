extern crate iron;
extern crate hyper;
extern crate time;
extern crate mount;
extern crate router;
extern crate staticfile;
extern crate unicase;
extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate image;
extern crate url;
extern crate cookie;

mod index;
mod template;
mod globals;
mod iron_ex;
mod headers;
mod release;
mod index_models;
mod resizer;
mod resizer_models;

use iron::prelude::*;
use iron::status;
use iron::method;
use std::convert::AsRef;
use std::collections::HashMap;

use hyper::header::{CacheControl, CacheDirective};
use hyper::header::Location;
use hyper::header::{ContentType, SetCookie, Cookie};
use cookie::Cookie as CookiePair;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use std::path::Path;
use std::sync::Mutex;
use std::sync::Arc;
use mount::Mount;
use staticfile::Static;
use router::Router;
use resizer::{ Resizer, ResizeMode, SizeHint };
use rand::distributions::{IndependentSample, Range};

fn send_page(index: &mut index::Index, resizer: &mut Resizer, req: &mut Request, cookie: &str) -> IronResult<Response> {
    match req.extensions.get::<Router>()
        .unwrap().find("slug") {
            Some(ref slug) => {
                match index.find(slug) {
                    Some(found) => {
                        let admin_access = match req.headers.get::<Cookie>() {
                            Some(&Cookie(ref vals)) => {
                                if vals.len() == 0 {
                                    false
                                } else {
                                    let maybe_cookie = vals.iter().find(|v| match *v {
                                        &CookiePair { ref name, .. } if name == "session" => true,
                                        _ => false,
                                    });
                                    match maybe_cookie {
                                        Some(&CookiePair { ref value, .. }) if value == cookie => true,
                                        _ => false,
                                    }
                                }
                            },
                            _ => false,
                        };

                        let image_url = match resizer.get_resized_url(
                            &found.file,
                            ResizeMode::Fit(
                                SizeHint { w: Some(1000), h: None }
                            ))
                        {
                            Some(i) => i.relative_url,
                            None => "".to_string(),
                        };

                        let mut vals = globals::Globals::new()
                            .with("title", found.title)
                            .with("file", ["/ic/", image_url.as_ref()].concat())
                            .with("width", "".into())
                            .with("height", "".into());

                        if admin_access {
                            let parsed = template::parse(
                                "views/admin/controls.html",
                                &vals
                            );
                            vals.amend("admin_controls", String::from_utf8_lossy(&parsed).into_owned());
                        } else {
                            vals.amend("admin_controls", String::new());
                        }

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

                        append_link(&mut vals, "random_disabled", "random_href", Some(
                            "/random".into()
                        ));

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

                        Ok(view("views/comic.html", vals))
                    },
                    None => Ok(Response::with(status::NotFound)),
                }
            },
            None => Ok(Response::with(status::NotFound)),
        }
}

static SYMBOLS: &'static [u8] = b"abcdefghijklmnopqrstuvyzABCDEFGHIJKLMNOPQRSTUVYZ1234567890";

fn random_str(len: u32) -> String {
    let between = Range::new(0, SYMBOLS.len());
    let mut rng = rand::thread_rng();

    let mut s = String::with_capacity(len as usize);
    for _ in 0..len {
        s.push(std::char::from_u32(SYMBOLS[between.ind_sample(&mut rng)] as u32).unwrap());
    }

    s
}

fn main() {
    // our "database", simply load from json file.
    let index = Arc::new(Mutex::new(index::Index::from_file("data/index.json")));
    let resizer = Mutex::new(Resizer::new(Path::new("data/images"), Path::new("cache/images")));
    let admin_cookie = random_str(120);

    println!("cookie: {:?}", admin_cookie);

    let index_for_pages = index.clone();
    let cookie_for_page = admin_cookie.clone();
    let mut router = Router::new();
    router
        .get("/:slug", move |req: &mut Request| -> IronResult<Response> {
            match (index_for_pages.lock(), resizer.lock()) {
                (Ok(mut index), Ok(mut resizer)) => {
                    send_page(&mut index, &mut resizer, req, &cookie_for_page)
                },
                _ => {
                    println!("Error locking index or resizer");
                    Ok(Response::with(status::NotFound))
                }
            }
        });

    let mut mount = Mount::new();
    let index_for_first_page = index.clone();
    let index_for_random = index.clone();
    mount
        .mount("/", move |_req: &mut Request| -> IronResult<Response> {
            match index_for_first_page.lock() {
                Ok(index) => {
                    match index.last_slug() {
                        Some(slug) => {
                            Ok(redirect(["/c/", slug.as_ref()].concat(), status::SeeOther))
                        },
                        None => {
                            println!("No pages exist");
                            Ok(Response::with(status::NotFound))
                        }
                    }
                },
                Err(e) => {
                    println!("Error locking index: {:?}", e);
                    Ok(Response::with(status::NotFound))
                }
            }
        })
        .mount("/c/", router)
        // .mount("/admin", admin_router)
        .mount("/favicon.png", Static::new(Path::new("public/favicon.png")))
        .mount("/ic/", Static::new(Path::new("cache/images")))
        .mount("/css/", Static::new(Path::new("public/css")))
        .mount("/js/", Static::new(Path::new("public/js")))
        .mount("/font/", Static::new(Path::new("public/font")))
        .mount("/i/", Static::new(Path::new("data/images")))
        .mount("/random", move |_req: &mut Request| -> IronResult<Response> {
            match index_for_random.lock() {
                Ok(index) => {
                    match index.random_slug() {
                        Some(slug) => {
                            Ok(redirect(["/c/", slug.as_ref()].concat(), status::SeeOther))
                        },
                        None => {
                            println!("No pages exist");
                            Ok(Response::with(status::NotFound))
                        },
                    }
                },
                Err(e) => {
                    println!("Error locking index: {:?}", e);
                    Ok(Response::with(status::NotFound))
                }
            }
        })
        .mount("/login", move |req: &mut Request| -> IronResult<Response> {
            println!("request method {:?}", req.method);
            match req.method {
                method::Method::Get => {
                    let mut vals = globals::Globals::new();

                    vals.amend("message", match req.url.query {
                        Some(_) => "<span class=\"error\">Enter valid username and password.</span>".into(),
                        None => "".into(),
                    });

                    Ok(view("views/login.html", vals))
                },
                _ => {
                    Ok(Response::with(status::NotFound))
                }
            }
        })
        .mount("/login-post", move |req: &mut Request| -> IronResult<Response> {
            println!("request method {:?}", req.method);
            let success = match req.url.query {
                Some(ref query) => {
                    let items = url::form_urlencoded::parse(query.as_bytes())
                        .into_iter()
                        .collect::<HashMap<_, _>>();

                    match (items.get("username").map(|v| v.as_ref()), items.get("password").map(|v| v.as_ref())) {
                        (Some("krabas"), Some("krabasseptyntaskis")) => true,
                        _ => false,
                    }
                },
                None => false,
            };

            if success {
                let mut response = redirect("/".into(), status::SeeOther);
                let mut cookie_pair = CookiePair::new("session".to_owned(), admin_cookie.to_owned());
                cookie_pair.expires = Some(time::now_utc() + time::Duration::hours(1));
                response.headers.set(
                   SetCookie(vec![
                       cookie_pair
                   ])
                );

                return Ok(response);
            }

            Ok(redirect("/login/?invalid".into(), status::SeeOther))
        })
    ;

    let mut chain = Chain::new(mount);

    chain = enable_browser_cache(chain);
    chain.link_after(iron_ex::not_found::NotFoundPage::new("views/not_found.html"));

    Iron::new(chain).http("localhost:3000").unwrap();
}

fn redirect(url: String, status: status::Status) -> Response {
    let mut response = Response::with(status);
    response.headers.set(
        Location(url)
    );
    response.headers.set(
        CacheControl(vec![
            CacheDirective::NoCache,
        ])
    );
    response
}

fn view(path: &str, vals: globals::Globals) -> Response {
    let parsed = template::parse(
        path,
        &vals
    );
    let mut response = Response::with((status::Ok, parsed));
    response.headers.set(
        ContentType(
            Mime(TopLevel::Text, SubLevel::Html, vec![(Attr::Charset, Value::Utf8)])
        )
    );
    response
}

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
