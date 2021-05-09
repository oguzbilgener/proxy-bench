use clap::Clap;
use socket2::{Domain, Socket, Type};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

#[macro_use]
extern crate lazy_static;

/// A simple TCP proxy
#[derive(Clap, Debug)]
struct Args {
    /// The address to listen on
    #[clap(short, long, default_value = "127.0.0.1:20000")]
    pub listen: String,
    /// The address to connect to
    #[clap(short, long, default_value = "127.0.0.1:20002")]
    pub upstream: String,
    /// Whether to use std copy util or custom implementation
    #[clap(short, long)]
    pub std_copy: bool,
    /// Buffer size for custom implementation
    #[clap(short, long, default_value = "1024")]
    pub buf_size: usize,
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

fn main() {
    println!(
        "std tcp server:: listen={}, upstream={}, std_copy={}, buf_size={}",
        &ARGS.listen, &ARGS.upstream, &ARGS.std_copy, &ARGS.buf_size,
    );
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();
    let address: SocketAddr = ARGS.listen.parse().unwrap();
    let _ = socket.bind(&address.into());
    let _ = socket.set_reuse_address(true);
    let _ = socket.listen(128);
    let listener: TcpListener = socket.into();
    // let listener = TcpListener::bind(&ARGS.listen).expect("Failed to bind to listen address");

    loop {
        let (socket, _) = listener.accept().unwrap();

        match TcpStream::connect(&ARGS.upstream) {
            Ok(target) => {
                std::thread::spawn(move || {
                    let mut cr = socket.try_clone().unwrap();
                    let mut cw = socket;
                    let mut ur = target.try_clone().unwrap();
                    let mut uw = target;

                    if ARGS.std_copy {
                        std::thread::spawn(move || {
                            let _ = std::io::copy(&mut cr, &mut uw);
                        });
                        std::thread::spawn(move || {
                            let _ = std::io::copy(&mut ur, &mut cw);
                        });
                    } else {
                        std::thread::spawn(move || {
                            let _ = forward(cr, uw);
                        });
                        std::thread::spawn(move || {
                            let _ = forward(ur, cw);
                        });
                    }
                });
            }
            Err(_) => {
                println!("Failed to connect to upstream.");
            }
        }
    }
}

fn forward(mut read: TcpStream, mut write: TcpStream) {
    let buf_size = ARGS.buf_size;
    let mut buf: Vec<u8> = vec![0; buf_size];
    while let Ok(n) = read.read(&mut buf) {
        if n == 0 || write.write_all(&buf[..n]).is_err() {
            break;
        }
    }
}
