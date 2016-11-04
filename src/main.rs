// #![deny(warnings)]
extern crate hyper;
extern crate docopt;
extern crate coroutine;
extern crate rustc_serialize;

use std::io::Write;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use docopt::Docopt;
use hyper::header::ContentLength;
use hyper::client::{Client, IntoUrl};

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

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_url: String,
    flag_c: usize,
    flag_d: usize,
    flag_t: usize,
    flag_v: bool,
}

macro_rules! t {
    ($e: expr) => (match $e {
        Err(err) => return println!("call = {:?}\nerr = {:?}", stringify!($e), err),
        Ok(val) => val,
    })
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_v {
        return println!("wrk: {}", VERSION);
    }

    let url = t!(args.arg_url.into_url());
    let test_conn_num = args.flag_c;
    let test_seconds = args.flag_d;

    let stop = AtomicBool::new(false);
    let total_req = AtomicUsize::new(0);
    let total_bytes = AtomicUsize::new(0);

    coroutine::scheduler_config().set_workers(2).set_io_workers(args.flag_t).set_stack_size(0x2000);
    coroutine::scope(|scope| {
        scope.spawn(|| {
            coroutine::sleep(Duration::from_secs(test_seconds as u64));
            stop.store(true, Ordering::Release);
        });

        // print the result every one second
        scope.spawn(|| {
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

                print!("\r{} Secs, Speed: {} requests/sec,  {} kb/sec\r",
                       time,
                       requests,
                       bytes / 1024);
                std::io::stdout().flush().ok();
            }
        });

        for _ in 0..test_conn_num {
            scope.spawn(|| {
                let mut client = Client::new();
                client.set_read_timeout(Some(Duration::from_secs(4)));
                loop {
                    let res = t!(client.get(url.clone()).send());
                    total_req.fetch_add(1, Ordering::Relaxed);
                    match res.headers.get::<ContentLength>() {
                        None => return println!("unable to get ContentLength"),
                        Some(&ContentLength(l)) => {
                            total_bytes.fetch_add(l as usize, Ordering::Relaxed);
                        }
                    }

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
    println!("Speed: {} request/sec, {} kb/sec",
             total_req / test_seconds,
             total_bytes / test_seconds / 1024);
    println!("Requests: {}", total_req);
}
