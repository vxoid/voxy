use std::net;
use std::io::{BufRead, Write};
use std::io;

pub struct Proxy {
    socket: net::TcpListener
}

impl Proxy {
    pub fn new(src: net::SocketAddr) -> io::Result<Proxy> {
        let socket = net::TcpListener::bind(src)?;

        Ok(Proxy { socket })
    }

    pub fn accept(&self) -> io::Result<()> {
        let (mut stream, _) = self.socket.accept()?;

        let buf_reader = io::BufReader::new(&mut stream);

        let http_request = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect::<Vec<String>>();

        let host_fields = http_request
            .iter()
            .filter(|field| field.starts_with("Host: "))
            .collect::<Vec<&String>>();
        let host_field = host_fields
            .first()
            .map_or(Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "no host was provided"
            )), |host| Ok(host))?;

        let host = host_field
            .strip_prefix("Host: ")
            .unwrap();

        let path = http_request[0].split(' ')
            .collect::<Vec<_>>()
            [1];
            
        let http_request = http_request[1..]
            .into_iter()
            .filter(|field| !field.starts_with("Host: "))
            .map(|field| {
                let field_slit = field
                    .split(": ")
                    .map(|str| str.to_string())
                    .collect::<Vec<_>>();

                (field_slit[0].clone(), field_slit[1].clone())
            })
            .collect::<Vec<_>>();

        let client = reqwest::blocking::Client::new();
        let mut request = client.get(format!("http://{}{}", host, path));

        for (key, value) in http_request {
            request = request.header(key, value)
        }

        let response = request
            .send()
            .map_err(|err| io::Error::new(
                io::ErrorKind::Interrupted,
                format!("got http error {}", err)
            ))?;

        stream.write_all(response_to_tcp_string(response).as_bytes())?;

        Ok(())
    }
}

fn response_to_tcp_string(response: reqwest::blocking::Response) -> String {
    let version = response.version();
    let status = response.status();
    let reason = status.canonical_reason().unwrap_or("Unknown");

    let mut tcp_string = format!("{:?} {} {}\r\n", version, status.as_str(), reason);

    for (header_name, header_value) in response.headers() {
        tcp_string = format!("{}{}: {:?}\r\n", tcp_string, header_name, header_value);
    }

    tcp_string = format!("{}\r\n", tcp_string);

    if let Ok(body) = response.text() {
        tcp_string.push_str(&body);
    }

    tcp_string
}