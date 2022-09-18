use std::io;

pub enum TcpState {
    Closed,
    Listen,
    SynRecv,
    Estab,
}

pub struct Connection {
    state: TcpState,
    send: SendSequenceSpace,
    receive: RecvSequenceSpace,
}

/// RFC (793 S3.2 F4)
///  Send Sequence Space
///
/// ```
///                   1         2          3          4      
///              ----------|----------|----------|----------
///                     SND.UNA    SND.NXT    SND.UNA        
///                                          +SND.WND        
///
///        1 - old sequence numbers which have been acknowledged  
///        2 - sequence numbers of unacknowledged data            
///        3 - sequence numbers allowed for new data transmission
///        4 - future sequence numbers which are not yet allowed  
/// ```
struct SendSequenceSpace {
    /// send unacknowledged
    una: usize,
    /// send next
    nxt: usize,
    /// send window
    wnd: usize,
    /// send urgent pointer
    up: bool,
    /// segment sequence number used for last window update
    wl1: usize,
    /// initial send sequence number
    wl2: usize,
}

/// Receive Sequence Space (RFC 793 S3.2 F5)
/// 
/// 1          2          3      
/// ----------|----------|---------- 
///        RCV.NXT    RCV.NXT        
///                  +RCV.WND        
/// 
/// 1 - old sequence numbers which have been acknowledged  
/// 2 - sequence numbers allowed for new reception         
/// 3 - future sequence numbers which are not yet allowed 
struct RecvSequenceSpace {
    /// receive next
    nxt: usize,
    /// receive window
    wnd: usize,
    /// receive urgent pointer
    up: bool,
    /// initial receive sequence number
    irs: usize,
}

impl Default for Connection {
    fn default() -> Self {
        // TcpState::Closed
        Connection {
            state: TcpState::Listen,
            send : ,
            receive : ,
        }
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
                return Ok(0);
            }
            TcpState::Listen => {
                if !tcph.syn() {
                    // only expected SYN packet
                    return Ok(0);
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
                    ip.write(&mut unwritten);
                    syn_ack.write(&mut unwritten);
                    unwritten.len()
                };

                nic.send(&buf[..unwritten]);
            }
            _ => {
                unimplemented!()
            }
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
