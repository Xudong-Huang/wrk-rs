
wrk
```sh
$ wrk http://127.0.0.1:8080 -d 10 -t 1 -c 200
Running 10s test @ http://127.0.0.1:8080
  1 threads and 200 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.06ms  538.79us  14.35ms   81.51%
    Req/Sec   112.03k     4.23k  117.68k    90.00%
  1115004 requests in 10.04s, 109.53MB read
Requests/sec: 111077.74
Transfer/sec:     10.91MB
wrk http://127.0.0.1:8080 -d 10 -t 1 -c 200  3.02s user 6.96s system 99% cpu 10.042 total
```

wrk-rs
```sh
$ ./target/release/wrk http://127.0.0.1:8080/ -d 10 -t 1 -c 200
==================Benchmarking: http://127.0.0.1:8080/==================
200 clients, 10 sec.

Speed: 76606 request/sec, 972 kb/sec
Requests: 766065
```