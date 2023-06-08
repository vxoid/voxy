mod proxy;

use proxy::Proxy;
use std::{net::*, str::FromStr};

fn usage(exe: &str) {
    let space = exe.chars()
        .map(|_| ' ')
        .collect::<String>();

    println!("{} 127.0.0.1:434", exe);
    println!("{} ^^^^^^^^^^^^^", space);
    println!("{} host to run proxy", space);
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let host = match args.get(1) {
        Some(host) => host,
        None => {
            println!("ERROR: host was not specified");
            usage(&args[0]);
            return;
        },
    };

    let proxy = Proxy::new(
        SocketAddr::from_str(host).expect(&format!("cannot parse {} as host", host))
    ).expect("proxy create error");

    loop {
        if let Err(_) = proxy.accept() {
            continue;
        }
    }
}