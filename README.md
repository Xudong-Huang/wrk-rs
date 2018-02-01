# wrk-rs

a simple [wrk](https://github.com/wg/wrk) implementation in rust

This is not fully optimized, it use the [may_http](https://github.com/rust-may/may_http) client implementation. Internally it will paring the headers and have some unnecessary work load.

## performance

Test against [may_minihttp](https://github.com/Xudong-Huang/may_minihttp) server with one io thread


wrk
```sh
$ wrk http://127.0.0.1:8080 -d 10 -t 1 -c 200
Running 10s test @ http://127.0.0.1:8080
  1 threads and 200 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.57ms  834.96us  17.79ms   96.60%
    Req/Sec   120.59k     6.54k  125.97k    89.00%
  1200361 requests in 10.05s, 117.91MB read
Requests/sec: 119460.72
Transfer/sec:     11.73MB
wrk http://127.0.0.1:8080 -d 10 -t 1 -c 200  2.82s user 7.10s system 98% cpu 10.067 total
```

wrk-rs

> Note that the Transfer rate is only for valid payload, not including headers.

```sh
$ cargo run --release -- http://127.0.0.1:8080/ -t 1 -c 200 -d 10
    Finished release [optimized] target(s) in 0.42 secs
     Running `target/release/wrk 'http://127.0.0.1:8080/' -t 1 -c 200 -d 10`
==================Benchmarking: http://127.0.0.1:8080/==================
200 clients, 10 sec.

Speed: 85470 request/sec, 1085 kb/sec
Requests: 854706
cargo run --release -- http://127.0.0.1:8080/ -t 1 -c 200 -d 10  4.55s user 5.89s system 94% cpu 11.077 total
```