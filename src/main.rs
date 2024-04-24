use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;
use etherparse::IpNumber;

mod tcp;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let mut nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buff = [0u8; 1504];

    loop {
        let nbytes = nic.recv(&mut buff[..])?;
        let _eth_flags = u16::from_be_bytes([buff[0], buff[1]]);
        let eth_proto = u16::from_be_bytes([buff[2], buff[3]]);
        if eth_proto != 0x0800 {
            // sem ipv4
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buff[4..nbytes]) {
            Ok(p) => {
                let src = p.source_addr();
                let dst = p.destination_addr();
                let protocol = p.protocol();

                if protocol != IpNumber::TCP {
                    // nao eh TCP
                    continue;
                }

                // Protocolo especifica que ate o 3, dados de flags e protocolo - hexadecimal
                let buffer_after_proto: usize = 4;

                // depois dos identificadores do ip
                let ip_header_size = p.slice().len();
                let data_position = buffer_after_proto + ip_header_size;

                // se houver dados no buffer apos os dados do ip
                match etherparse::TcpHeaderSlice::from_slice(&buff[data_position..nbytes]) {
                    Ok(tcph) => {
                        let tcp_data_position = data_position + tcph.slice().len();
                        connections.entry(Quad {
                            src: (src, tcph.source_port()),
                            dst: (dst, tcph.destination_port()),
                        }).or_default().on_packet(&mut nic, p, tcph, &buff[tcp_data_position..nbytes])?;
                    }
                    Err(e) => {
                        eprintln!("Ignorando pacote estranho {:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error in read of Ipv4 connection - ignoring weird package {:?}", e);
            }
        }
    }
}
