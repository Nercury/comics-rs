use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;

pub struct Time;

impl typemap::Key for Time { type Value = u64; }

impl BeforeMiddleware for Time {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Time>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for Time {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<Time>().unwrap();
        //println!("{:?}", res.status);
        println!("{} ms, {} {}", (delta as f64) / 1000000.0, req.method, req.url);
        Ok(res)
    }
}
