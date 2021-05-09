use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion, Throughput,
};
use std::io;
use std::process::{Child, Command};
use tokio::runtime::Runtime;

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn make_test_http_server_cmd(listen: &str) -> io::Result<Child> {
    Command::new("../testserver/target/release/testserver")
        .arg("--listen")
        .arg(format!("127.0.0.1:{}", listen))
        .spawn()
}

fn make_go_proxy_cmd(listen: &str, upstream: &str) -> io::Result<Child> {
    Command::new("../go_tcp_proxy/go_tcp_proxy")
        .arg("-listen")
        .arg(format!("127.0.0.1:{}", listen))
        .arg("-upstream")
        .arg(format!("127.0.0.1:{}", upstream))
        .spawn()
}

fn make_tokio_proxy_cmd(
    listen: &str,
    upstream: &str,
    use_copy: bool,
    use_copy_bi: bool,
    buf_size: &str,
    thread_count: usize,
) -> io::Result<Child> {
    let mut cmd = Command::new("../tokio_tcp_proxy/target/release/tokio_tcp_proxy");
    let child = cmd
        .arg("--thread-count")
        .arg(thread_count.to_string())
        .arg("--listen")
        .arg(format!("127.0.0.1:{}", listen))
        .arg("--upstream")
        .arg(format!("127.0.0.1:{}", upstream));
    let child = if use_copy {
        child.arg("--tokio-copy")
    } else if use_copy_bi {
        child.arg("--tokio-copy-bi")
    } else {
        child.arg("--buf-size").arg(buf_size)
    };
    child.spawn()
}

struct Handle(Child);

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

fn run_concurrent_command_until_stop<F>(make_command: F) -> io::Result<Handle>
where
    F: Fn() -> io::Result<Child>,
{
    Ok(Handle(make_command()?))
}

fn with_server<F, T, P>(
    group: &mut BenchmarkGroup<WallTime>,
    mut task: F,
    make_target_command: T,
    make_proxy_command: P,
) where
    F: FnMut(&mut BenchmarkGroup<WallTime>, &Runtime),
    T: Fn() -> io::Result<Child>,
    P: Fn() -> io::Result<Child>,
{
    let rt = rt();

    let _ = run_concurrent_command_until_stop(make_target_command)
        .and_then(|h1| run_concurrent_command_until_stop(make_proxy_command).map(|h2| (h1, h2)))
        .map(|_| task(group, &rt))
        .or_else(|err| {
            println!("Failed with error: {}", err);
            Ok::<(), io::Error>(())
        });
}

fn benchmark_http_1(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_http_1");
    group.throughput(Throughput::Elements(1u64));

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("direct", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20004/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20004"),
        || make_go_proxy_cmd("20003", "20004"),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("go", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20003/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20004"),
        || make_go_proxy_cmd("20003", "20004"),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio 32K buffer, 16 threads", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, false, "32768", 16),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio 32K buffer, 1 thread", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, false, "32768", 1),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio 64K buffer, 1 thread", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, false, "65536", 1),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio 1M buffer, 1 thread", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, false, "1048576", 1),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio 8K buffer, 16 threads", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, false, "8192", 16),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio with tokio::io::copy (2K buffer), 16 threads", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", true, false, "0", 16),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio with tokio::io::copy (2K buffer), 1 thread", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", true, false, "0", 1),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio with tokio::io::copy_bidirectional (2K buffer), 1 thread", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test1").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, true, "0", 1),
    );
}

fn benchmark_http_2(c: &mut Criterion) {
    let mut group = c.benchmark_group("benchmark_http_2");
    group.throughput(Throughput::Elements(1u64));

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("direct", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20004/test2").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20004"),
        || make_go_proxy_cmd("20003", "20004"),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("go", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20003/test2").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20004"),
        || make_go_proxy_cmd("20003", "20004"),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio 32K buffer, 16 threads", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test2").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, false, "32768", 16),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio 32K buffer, 1 thread", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test2").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, false, "32768", 1),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio 8K buffer", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test2").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", false, false, "8192", 16),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio with tokio::io::copy (2K buffer), 16 threads", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test2").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", true, false, "0", 16),
    );

    with_server(
        &mut group,
        move |group, rt| {
            group.bench_function("tokio with tokio::io::copy (2K buffer), 1 thread", |b| {
                let client = reqwest::Client::new();
                b.iter(|| {
                    let client = client.clone();
                    rt.block_on(async move {
                        let res = client.get("http://127.0.0.1:20000/test2").send().await;
                        match res {
                            Ok(r) => {
                                let _ = r.text().await;
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    });
                });
            });
        },
        || make_test_http_server_cmd("20001"),
        || make_tokio_proxy_cmd("20000", "20001", true, false, "0", 1),
    );
}

criterion_group!(benches, benchmark_http_1, benchmark_http_2);
criterion_main!(benches);
