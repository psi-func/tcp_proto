use std::io;

pub enum TcpState {
    Closed,
    Listen,
    SynRecv,
    Estab,
}

impl Default for TcpState {
    fn default() -> Self {
        // TcpState::Closed
        TcpState::Listen
    }
}

impl TcpState {
    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        _buf: &'a [u8],
    ) -> io::Result<usize> {
        let mut buf = [0u8; 1500];

        match *self {
            TcpState::Closed => {
                return;
            }
            TcpState::Listen => {
                if !tcph.syn() {
                    // only expected SYN packet
                    return;
                }

                // need to start establishing a connection
                let mut syn_ack = etherparse::TcpHeader::new(
                    tcph.destination_port(),
                    tcph.source_port(),
                    unimplemented!(),
                    unimplemented!(),
                );
                syn_ack.syn = true;
                syn_ack.ack = true;

                let mut ip = etherparse::Ipv4Header::new(
                    syn_ack.header_len(),
                    64,
                    etherparse::ip_number::TCP,
                    iph.destination(),
                    iph.source(),
                );

                // write out headers
                let unwritten = {
                    let mut unwritten = &mut buf[..];
                    ip.write(unwritten);
                    syn_ack.write(unwritten);
                    unwritten.len()
                }

                nic.send(&buf[..unwritten]);
            }
            _ => {}
        }

        // println!(
        //     "{}:{} -> {}:{} {}b of tcp",
        //     iph.source_addr(),
        //     tcph.source_port(),
        //     iph.destination_addr(),
        //     tcph.destination_port(),
        //     tcph.slice().len(),
        // );
    }
}
