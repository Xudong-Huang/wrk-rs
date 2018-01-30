#![deny(warnings)]
extern crate docopt;
#[macro_use]
extern crate may;
extern crate may_http;
#[macro_use]
extern crate serde_derive;

use std::time::Duration;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use docopt::Docopt;
use may::coroutine;
use may_http::http::Uri;
use may_http::client::HttpClient;

const VERSION: &'static str = "0.1.0";
const USAGE: &'static str = "
Wrk.

Usage:
  Wrk [-t <threads>] [-c <connections>] [-d <time>] <url>
  Wrk (-h | --help)
  Wrk (-v | --version)

Options:
  -h --help         Show this screen.
  -v --version      Show version.
  -t <threads>      number of threads to use [default: 1].
  -c <connections>  concurent connections  [default: 100].
  -d <time>         time to run in seconds [default: 10].
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_url: String,
    flag_c: usize,
    flag_d: usize,
    flag_t: usize,
    flag_v: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_v {
        return println!("wrk: {}", VERSION);
    }

    // let url = t!(args.arg_url.into_url());
    let url: Uri = args.arg_url.parse().unwrap();
    let remote = url.host().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(80);
    let test_conn_num = args.flag_c;
    let test_seconds = args.flag_d;

    let stop = AtomicBool::new(false);
    let total_req = AtomicUsize::new(0);
    let total_bytes = AtomicUsize::new(0);

    may::config().set_workers(1).set_io_workers(args.flag_t);
    coroutine::scope(|scope| {
        go!(scope, || {
            coroutine::sleep(Duration::from_secs(test_seconds as u64));
            stop.store(true, Ordering::Release);
        });

        // print the result every one second
        go!(scope, || {
            let mut time = 0;
            let mut last_request = 0;
            let mut last_bytes = 0;
            while !stop.load(Ordering::Relaxed) {
                coroutine::sleep(Duration::from_secs(1));
                time += 1;

                let total_req = total_req.load(Ordering::Relaxed);
                let requests = total_req - last_request;
                last_request = total_req;

                let total_bytes = total_bytes.load(Ordering::Relaxed);
                let bytes = total_bytes - last_bytes;
                last_bytes = total_bytes;

                print!(
                    "\r{} Secs, Speed: {} requests/sec,  {} kb/sec\r",
                    time,
                    requests,
                    bytes / 1024
                );
                std::io::stdout().flush().ok();
            }
        });

        for _ in 0..test_conn_num {
            go!(scope, || {
                let mut buf = vec![0; 4096];
                let mut client = HttpClient::connect(&(remote, port)).unwrap();
                loop {
                    // client.set_timeout(Some(Duration::from_secs(4)));
                    let mut rsp = client.get(url.clone()).unwrap();

                    let recv_bytes = rsp.read_to_end(&mut buf).unwrap();
                    total_req.fetch_add(1, Ordering::Relaxed);
                    total_bytes.fetch_add(recv_bytes, Ordering::Relaxed);

                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                }
            });
        }
    });

    let total_req = total_req.load(Ordering::Relaxed);
    let total_bytes = total_bytes.load(Ordering::Relaxed);

    println!("==================Benchmarking: {}==================", url);
    println!("{} clients, {} sec.\n", test_conn_num, test_seconds);
    println!(
        "Speed: {} request/sec, {} kb/sec",
        total_req / test_seconds,
        total_bytes / test_seconds / 1024
    );
    println!("Requests: {}", total_req);
}
