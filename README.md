### proxy-bench

This is a benchmark that compares a few different basic TCP proxy implementations in Go and Rust.

`prepare-and-run.sh` builds and executes the benchmark and it requires Go and Rust toolchains.

The benchmark measures the latency of the proxy implementations by sending HTTP requests over a TCP connection repeatedly.

There are two test cases:

1. Make a HTTP/1 GET request to /test1, which returns a 64K-long response body.
2. Make a HTTP/1 GET request to /test2, which returns a simple "Hello, world!" response.

Below is the output of a sample run on Ubuntu 20.04 running on AMD Ryzen 9 3950X.

-   Go version: 1.16.4
-   Rust version: 1.52.1
-   Tokio version: 1.6.0

The Go implementation uses `io.Copy` which has an 32K internal buffer. The std and Tokio-based Rust implementations offer a command-line argument to set the buffer size.

| Test   | Case                                                            | Time      | Throughput     |
| ------ | --------------------------------------------------------------- | --------- | -------------- |
| Test 1 | Direct connection                                               | 99.608 us | 10.04 Kelem/s  |
| Test 1 | Go proxy                                                        | 122.01 us | 8.19 Kelem/s   |
| Test 1 | Tokio (32K buffer, 16 threads)                                  | 190.49 us | 5.24 Kelem/s   |
| Test 1 | Tokio (32K buffer, 1 thread)                                    | 181.75 us | 5.50 Kelem/s   |
| Test 1 | Tokio (64K buffer, 1 thread)                                    | 136.45 us | 7.32 Kelem/s   |
| Test 1 | Tokio (1M buffer, 1 thread)                                     | 133.63 us | 7.48 Kelem/s   |
| Test 1 | Tokio (8K buffer, 16 threads)                                   | 20.219 ms | 49.45 elem/s   |
| Test 1 | Tokio (w/ `tokio::io::copy`, 2K buffer, 16 threads)             | 22.774 ms | 43.91 elem/s   |
| Test 1 | Tokio (w/ `tokio::io::copy`, 2K buffer, 1 thread)               | 25.373 ms | 39.39 elem/s   |
| Test 1 | Tokio (w/ `tokio::io::copy_bidirectional`, 2K buffer, 1 thread) | 28.190 ms | 35.47 elem/s   |
| Test 1 | std (64K buffer)                                                | 134.46 us | 7.43 Kelem/s   |
| Test 1 | std (w/ `std::io::copy`, 8K buffer?)                            | 31.662 us | 31.58 Kelem/s  |
| Test 2 | Direct connection                                               | 53.412 us | 18.72 Kelem/s  |
| Test 2 | Go proxy                                                        | 80.494 us | 12.423 Kelem/s |
| Test 2 | Tokio (32K buffer, 16 threads)                                  | 86.734 us | 11.53 Kelem/s  |
| Test 2 | Tokio (32K buffer, 1 thread)                                    | 78.640 us | 12.71 Kelem/s  |
| Test 2 | Tokio (8K buffer, 16 threads)                                   | 85.59 us  | 11.68 Kelem/s  |
| Test 2 | Tokio (w/ `tokio::io::copy`, 2K buffer, 16 threads)             | 80.99 us  | 12.34 Kelem/s  |
| Test 2 | Tokio (w/ `tokio::io::copy`, 2K buffer, 1 thread)               | 79.39 us  | 12.59 Kelem    |
| Test 2 | std (2K buffer)                                                 | 78.63 us  | 12.71 Kelem/s  |
| Test 2 | std (w/ `std::io::copy`, 8K buffer?)                            | 77.680 us | 12.873 Kelem/s |
