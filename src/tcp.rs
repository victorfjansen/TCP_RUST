enum State {
    Closed,
    Listen,
    //SynRcvd,
    //Estab,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: ReceiveSequenceSpace
}


///                Send Sequence Space (RFC 793 - 3.2)
///
///                   1         2          3          4
///              ----------|----------|----------|----------
///                     SND.UNA    SND.NXT    SND.UNA
///                                          +SND.WND
///
///        1 - old sequence numbers which have been acknowledged
///        2 - sequence numbers of unacknowledged data
///        3 - sequence numbers allowed for new data transmission
///        4 - future sequence numbers which are not yet allowed
///
///                          Send Sequence Space

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
    /// segment acknowledgment number used for last window update
    wl2: usize,
    /// initial send sequence number
    iss: usize
}

///  Receive Sequence Space
///
///                        1          2          3
///                    ----------|----------|----------
///                           RCV.NXT    RCV.NXT
///                                     +RCV.WND
///
///        1 - old sequence numbers which have been acknowledged
///         2 - sequence numbers allowed for new reception
///         3 - future sequence numbers which are not yet allowed
///
///                          Receive Sequence Space

struct ReceiveSequenceSpace {
    /// receive next
    nxt: usize,
    /// receive window
    wnd: usize,
    /// receive urgent pointer
    up: usize,
    /// initial receive sequence number
    irs: usize
}

impl Default for Connection {
    fn default() -> Self {
        // State::Closed
        Connection {
            state: State::Listen
        }
    }
}

impl Connection {
    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> std::io::Result<usize> {
        let mut buff = [0u8; 1504];
        match *self {
            State::Closed => {
                return Ok(0);
            }
            State::Listen => {
                if !tcph.syn() {
                    // apenas esperamos um pacote SYN;
                    return Ok(0);
                }

                // estabelecendo conexao
                // conexao de retorno: source_port vira destino (conexao original) e destination_port vira origem
                let mut syn_ack = etherparse::TcpHeader::new(tcph.destination_port(), tcph.source_port(), 0, 0);
                syn_ack.syn = true;
                syn_ack.ack = true;

                let mut ip =
                    etherparse::Ipv4Header::new(syn_ack.header_len_u16(),
                                                64,
                                                    etherparse::IpNumber::TCP,
                                                [
                                                    iph.destination()[0],
                                                    iph.destination()[1],
                                                    iph.destination()[2],
                                                    iph.destination()[3]
                                                ],
                                                [
                                                    iph.source()[0],
                                                    iph.source()[1],
                                                    iph.source()[2],
                                                    iph.source()[3],
                                                ],
                    ).ok().unwrap();

                let unwritten = {
                    let mut unwritten = &mut buff[..];
                    ip.write(&mut unwritten)?;
                    syn_ack.write(&mut unwritten)?;
                    unwritten.len()
                };

                nic.send(&buff[..unwritten])
            }
            // State::Estab () => {},
            // State::SynRcvd () => {},
        }
    }
}