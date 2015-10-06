use iron::prelude::*;
use iron::{AfterMiddleware, typemap};
use iron::status::Status;
use hyper::header::{ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use template;
use globals;

pub struct NotFoundPage {
    path: String,
}

impl NotFoundPage {
    pub fn new(template_path: &str) -> NotFoundPage {
        NotFoundPage {
            path: template_path.into(),
        }
    }
}

impl typemap::Key for NotFoundPage { type Value = u64; }

impl AfterMiddleware for NotFoundPage {
    fn after(&self, _req: &mut Request, mut res: Response) -> IronResult<Response> {
        match res.status {
            Some(Status::NotFound) => (),
            _ => return Ok(res),
        }

        let parsed = template::parse(
            &self.path,
            globals::Globals::new()
        );

        res.body = Some(Box::new(parsed));
        res.headers.set(
            ContentType(
                Mime(TopLevel::Text, SubLevel::Html, vec![(Attr::Charset, Value::Utf8)])
            )
        );

        Ok(res)
    }
}
