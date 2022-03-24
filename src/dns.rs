use std::io::{BufReader, Write};
use thiserror::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use mio::net::UdpSocket;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{ BufMut };
use mio::{Events, Interest, Poll, Token};

pub struct DnsClient {
    server_host: Ipv4Addr,
    server_port: u16
}

#[derive(Error, Debug)]
pub enum DnsClientError {
    #[error("Unknown hostname")]
    UnknownHostname,

    #[error("IO Error")]
    IoError(#[from] std::io::Error),
}

impl DnsClient {
    pub fn new(server_host: Ipv4Addr, server_port: u16) -> DnsClient {
        DnsClient {
            server_host,
            server_port,
        }
    }

    pub async fn resolve(&self, hostname: String) -> Result<Ipv4Addr, DnsClientError> {
        let mut socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), 0))?;
        socket.connect(SocketAddr::new(IpAddr::V4(self.server_host), self.server_port))?;

        let buffer: Vec<u8> = vec![];
        let mut writer = buffer.writer();

        writer.write_u16::<BigEndian>(1u16).unwrap(); // identification
        writer.write_u16::<BigEndian>(0u16).unwrap(); // flags
        writer.write_u16::<BigEndian>(1u16).unwrap(); // number_of_questions
        writer.write_u16::<BigEndian>(0u16).unwrap(); // number_of_answers
        writer.write_u16::<BigEndian>(0u16).unwrap(); // number_of_authority_prs
        writer.write_u16::<BigEndian>(0u16).unwrap(); // number_of_additional_prs

        hostname
            .split(".")
            .for_each(|str| {
                writer.write_u8(str.len() as u8).unwrap();
                writer.write_all(str.as_bytes()).unwrap();
            });

        writer.write_u8(0).unwrap();

        writer.write_u16::<BigEndian>(1).unwrap();
        writer.write_u16::<BigEndian>(1).unwrap();

        socket.send(writer.get_ref().as_slice())?;

        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(10);

        const SOCK: Token = Token(1);

        poll.registry().register(&mut socket, SOCK, Interest::READABLE).unwrap();
        poll.poll(&mut events, None).unwrap();

        let mut buffer = vec![0; 512];
        socket.recv(buffer.as_mut_slice())?;
        self.extract_response(buffer.as_slice())
    }

    fn extract_response(&self, data: &[u8]) -> Result<Ipv4Addr, DnsClientError> {
        let mut reader = BufReader::new(data);
        reader.read_u16::<BigEndian>().unwrap();
        reader.read_u16::<BigEndian>().unwrap();
        let number_of_questions = reader.read_u16::<BigEndian>().unwrap();
        let number_of_answers = reader.read_u16::<BigEndian>().unwrap();
        reader.read_u16::<BigEndian>().unwrap();
        reader.read_u16::<BigEndian>().unwrap();

        for _ in 0..number_of_questions {
            loop {
                let c = reader.read_u8().unwrap() as usize;
                if c == 0 { break };
                for _ in 0..c {
                    reader.read_u8().unwrap();
                }
            }

            reader.read_u16::<BigEndian>().unwrap();
            reader.read_u16::<BigEndian>().unwrap();
        }

        if number_of_answers == 0 {
            return Err(DnsClientError::UnknownHostname);
        }

        reader.read_u16::<BigEndian>().unwrap(); //name
        reader.read_u16::<BigEndian>().unwrap(); // type
        reader.read_u16::<BigEndian>().unwrap(); // class
        reader.read_u32::<BigEndian>().unwrap(); //TTL
        reader.read_u16::<BigEndian>().unwrap(); //len

        let a = reader.read_u8().unwrap();
        let b = reader.read_u8().unwrap();
        let c = reader.read_u8().unwrap();
        let d = reader.read_u8().unwrap();

        Ok(Ipv4Addr::new(a,  b, c, d))
    }
}
