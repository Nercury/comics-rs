use iron::prelude::*;
use iron::{AfterMiddleware};
use unicase::UniCase;

use hyper::header::{Vary, ContentType, CacheControl, CacheDirective};
use hyper::mime::{Mime, TopLevel, SubLevel};

#[allow(dead_code)]
pub struct StaticHeaders;

impl AfterMiddleware for StaticHeaders {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {

        let is_static_file = match res.headers.get::<ContentType>() {
            Some(&ContentType(Mime(TopLevel::Text, SubLevel::Css, _))) => true,
            Some(&ContentType(Mime(TopLevel::Application, SubLevel::Javascript, _))) => true,
            Some(&ContentType(Mime(TopLevel::Image, SubLevel::Ext(ref kind), _))) => {
                match &kind[..] {
                    "svg+xml" => true,
                    other => {
                        println!("other image: {:?}", other);
                        false
                    },
                }
            },
            Some(&ContentType(Mime(TopLevel::Application, SubLevel::Ext(ref kind), _))) => {
                match &kind[..] {
                    "font-woff" | "x-font-ttf" | "vnd.ms-fontobject" => true,
                    other => {
                        println!("other font: {:?}", other);
                        false
                    },
                }
            },
            _ => false,
        };

        if is_static_file {
            res.headers.set(
                CacheControl(vec![
                    CacheDirective::Public,
                    CacheDirective::MaxAge(2592000u32),
                ])
            );
            res.headers.set(
                Vary::Items(vec![
                    UniCase("accept-encoding".to_owned()),
                ])
            );
        }

        Ok(res)
    }
}
