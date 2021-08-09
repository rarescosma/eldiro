use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();

    let mut nic = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1500];
    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        // let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        // let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        //
        // if eth_proto != 0x0800 {
        //     // only ipv4
        //     continue;
        // }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                let proto = iph.protocol();
                if proto != 0x06 {
                    // only tcp
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice(&buf[iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        let data_index = iph.slice().len() + tcph.slice().len();
                        match connections.entry(Quad {
                            src: (src, tcph.source_port()),
                            dst: (dst, tcph.destination_port()),
                        }) {
                            Entry::Occupied(mut c) => {
                                // let foo = c.get();
                                c.get_mut().on_packet(
                                    &mut nic,
                                    iph,
                                    tcph,
                                    &buf[data_index..nbytes],
                                );
                            }
                            Entry::Vacant(e) => {
                                if let Some(c) = tcp::Connection::accept(
                                    &mut nic,
                                    iph,
                                    tcph,
                                    &buf[data_index..nbytes],
                                )? {
                                    e.insert(c);
                                }
                            }
                        }
                        // .or_default()
                        //
                    }
                    Err(e) => eprintln!("ignoring weird packet {:?}", e),
                }
            }
            Err(e) => eprintln!("ignoring weird packet {:?}", e),
        }
    }
}
