use clap::Clap;
use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use std::{
    convert::Infallible,
    mem::{self, MaybeUninit},
};

#[macro_use]
extern crate lazy_static;

/// A simple TCP proxy
#[derive(Clap, Debug)]
struct Args {
    /// The address to listen on
    #[clap(short, long, default_value = "127.0.0.1:20002")]
    pub listen: String,
}

const FIRST_SIZE: usize = 64 * 1024;
const FIRST_BIN_SIZE: usize = 32 * 1024;

struct Data {
    first: [u8; FIRST_SIZE],
    second: [u8; 11],
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "listen={}", self.listen)
    }
}

impl Data {
    fn new() -> Self {
        // Build 64k payload made of hex chars
        let first: [u8; FIRST_SIZE] = unsafe {
            let first: [MaybeUninit<u8>; FIRST_SIZE] = MaybeUninit::uninit().assume_init();
            let mut stuff: [MaybeUninit<u8>; FIRST_BIN_SIZE] = MaybeUninit::uninit().assume_init();
            #[allow(clippy::needless_range_loop)]
            for i in 0..FIRST_BIN_SIZE {
                stuff[i] = MaybeUninit::new((i % 256) as u8);
            }
            let stuff = mem::transmute::<_, [u8; FIRST_BIN_SIZE]>(stuff);
            let mut first = mem::transmute::<_, [u8; FIRST_SIZE]>(first);
            hex::encode_to_slice(stuff, &mut first).expect("Could not encode data to hex");
            first
        };

        Data {
            first,
            second: b"Hello world".to_owned(),
        }
    }
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
    static ref DATA: Data = Data::new();
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match req.uri().path() {
        "/test1" => {
            let r: &'static [u8] = &DATA.first;
            Ok(Response::new(r.into()))
        }
        "/test2" => {
            let r: &'static [u8] = &DATA.second;
            Ok(Response::new(r.into()))
        }
        _ => Ok(Response::builder()
            .status(404)
            .body("Not found".into())
            .unwrap()),
    }
}

#[tokio::main]
async fn main() {
    let addr = ARGS
        .listen
        .parse()
        .expect("Could not parse listen address to SocketAddr");

    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(handle)) });

    let server = Server::bind(&addr).serve(service);

    println!("Testserver listening on http://{}", addr);

    server.await.unwrap();
}
