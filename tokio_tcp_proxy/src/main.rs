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
    /// Buffer size for custom implementation
    #[clap(short, long, default_value = "1024")]
    pub buf_size: usize,
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "listen={}, upstream={}, tokio_copy={}, buf_size={}",
            self.listen, self.upstream, self.tokio_copy, self.buf_size
        )
    }
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!(
        "listen={}, upstream={}, tokio_copy={}, buf_size={}",
        &ARGS.listen, &ARGS.upstream, ARGS.tokio_copy, ARGS.buf_size
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
                Ok(tar) => {
                    let (mut client_read, mut client_write) = socket.into_split();
                    let (mut upstream_read, mut upstream_write) = tar.into_split();
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
                Err(_) => {
                    println!("Failed to connect to upstream.");
                }
            }
        });
    }
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
