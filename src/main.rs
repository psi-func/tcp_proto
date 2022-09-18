mod tcp;

use tcp::TcpState;

use etherparse::{
    self, ip_number, Ipv4Header, SerializedSize, TcpHeaderSlice,
};
use tun_tap::Iface;

use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst_ip: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, TcpState> = Default::default();
    let mut nic = Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buffer = vec![0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buffer).unwrap();
        let _eth_flags = u16::from_be_bytes([buffer[0], buffer[1]]);
        let eth_proto = u16::from_be_bytes([buffer[2], buffer[3]]);
        if eth_proto != 0x0800 {
            // no ipv4
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buffer[4..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                let proto = iph.protocol();
                if proto != ip_number::TCP {
                    // not tcp
                    continue;
                }

                match TcpHeaderSlice::from_slice(&buffer[(4 + Ipv4Header::SERIALIZED_SIZE)..nbytes])
                {
                    Ok(tcph) => {
                        let datai = 4 + Ipv4Header::SERIALIZED_SIZE + tcph.slice().len();
                        connections
                            .entry(Quad {
                                src: (src, tcph.source_port()),
                                dst_ip: (dst, tcph.destination_port()),
                            })
                            .or_default()
                            .on_packet(&mut nic, iph, tcph, &buffer[datai..nbytes]);
                    }
                    Err(_) => {
                        eprintln!("Bad TCP header");
                    }
                }
            }
            Err(_) => {
                eprintln!("Bad format");
            }
        }
    }
    Ok(())
}
