use clap::Clap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
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
    /// Whether to use tokio copy util or custom implementation
    #[clap(short, long)]
    pub tokio_copy: bool,
    /// Whether to use tokio copy_bidirectional
    #[clap(long)]
    pub tokio_copy_bi: bool,
    /// Buffer size for custom implementation
    #[clap(short, long, default_value = "1024")]
    pub buf_size: usize,
    #[clap(short, long, default_value = "6")]
    pub thread_count: usize,
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

async fn listen() {
    println!(
        "listen={}, upstream={}, tokio_copy={}, tokio_copy_bi={}, buf_size={}",
        &ARGS.listen, &ARGS.upstream, ARGS.tokio_copy, ARGS.tokio_copy_bi, ARGS.buf_size
    );
    let listener = TcpListener::bind(&ARGS.listen)
        .await
        .expect("Failed to bind to listen address");

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("Failed to accept a new connection");

        tokio::spawn(async move {
            match TcpStream::connect(&ARGS.upstream).await {
                Ok(mut target) => {
                    if ARGS.tokio_copy_bi {
                        let mut socket = socket;
                        let _ = tokio::io::copy_bidirectional(&mut target, &mut socket).await;
                    } else {
                        let (mut client_read, mut client_write) = socket.into_split();
                        let (mut upstream_read, mut upstream_write) = target.into_split();
                        let upstream_handle = tokio::spawn(async move {
                            if ARGS.tokio_copy {
                                let _ = tokio::io::copy(&mut client_read, &mut upstream_write).await;
                            } else {
                                forward_custom(client_read, upstream_write).await;
                            }
                        });
                        let downstream_handle = tokio::spawn(async move {
                            if ARGS.tokio_copy {
                                let _ = tokio::io::copy(&mut upstream_read, &mut client_write).await;
                            } else {
                                forward_custom(upstream_read, client_write).await;
                            }
                        });

                        let _ = upstream_handle.await;
                        let _ = downstream_handle.await;
                    }
                }
                Err(_) => {
                    println!("Failed to connect to upstream.");
                }
            }
        });
    }
}

fn main() {
    let runtime = if ARGS.thread_count > 1 {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(ARGS.thread_count)
            .enable_all()
            .build()
            .unwrap()
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    };

    runtime.block_on(listen());
}

async fn forward_custom(mut read: OwnedReadHalf, mut write: OwnedWriteHalf) {
    let buf_size = ARGS.buf_size;
    let mut buf: Vec<u8> = vec![0; buf_size];
    while let Ok(n) = read.read(&mut buf).await {
        if n == 0 || write.write_all(&buf[..n]).await.is_err() {
            break;
        }
    }
}
