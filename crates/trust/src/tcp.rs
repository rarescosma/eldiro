use std::io;

pub enum State {
    // Listen,
    Closed,
    SyncRcvd,
    Estab,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
    ip: etherparse::Ipv4Header,
}

struct SendSequenceSpace {
    /// send unacknowledged
    una: u32,
    /// send next
    nxt: u32,
    /// send window
    wnd: u16,
    /// send urgent pointer
    up: bool,
    /// segment seq number used for last window update
    wl1: usize,
    /// segment ack number used for last window update
    wl2: usize,
    /// initial send sequence number
    iss: u32,
}

struct RecvSequenceSpace {
    /// receive next
    nxt: u32,
    /// receive window
    wnd: u16,
    /// receive urgent pointer
    up: bool,
    /// initial receive sequence number
    irs: u32,
}

impl Connection {
    pub fn accept(
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) -> io::Result<Option<Self>> {
        let mut buf = [0u8; 1500];

        if !tcph.syn() {
            // only expected SYN packet
            return Ok(None);
        }

        let iss = 0;
        let mut c = Connection {
            state: State::SyncRcvd,
            send: SendSequenceSpace {
                iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,

                wl1: 0,
                wl2: 0,
            },
            recv: RecvSequenceSpace {
                irs: tcph.sequence_number(),
                nxt: tcph.sequence_number() + 1,
                wnd: tcph.window_size(),
                up: false,
            },
            ip: etherparse::Ipv4Header::new(
                0,
                64,
                etherparse::IpTrafficClass::Tcp,
                [
                    iph.destination()[0],
                    iph.destination()[1],
                    iph.destination()[2],
                    iph.destination()[3],
                ],
                [
                    iph.source()[0],
                    iph.source()[1],
                    iph.source()[2],
                    iph.source()[3],
                ],
            ),
        };

        // need to start establishing a conn
        let mut syn_ack = etherparse::TcpHeader::new(
            tcph.destination_port(),
            tcph.source_port(),
            c.send.iss,
            c.send.wnd,
        );
        syn_ack.acknowledgment_number = c.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;
        c.ip.set_payload_len(syn_ack.header_len() as usize);
        syn_ack.checksum = syn_ack
            .calc_checksum_ipv4(&c.ip, &[])
            .expect("failed to compute checksum");

        let unwritten = {
            let mut unwritten = &mut buf[..];
            c.ip.write(&mut unwritten);
            syn_ack.write(&mut unwritten);
            unwritten.len()
        };

        nic.send(&buf[..buf.len() - unwritten])?;
        Ok(Some(c))
    }

    pub fn on_packet(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice,
        tcph: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) -> io::Result<()> {
        // acceptable ack check
        // SND.UNA < SEG.ACK =< SND.NXT
        let seg_ack = tcph.acknowledgment_number();
        if !s_lt_x_lte_e_wrapping(self.send.una, seg_ack, self.send.nxt) {
            return Ok(());
        }

        // valid segment check - okay if it acks at least one byte
        // either:
        //   RCV.NXT =< SEG.SEQ           < RCV.NXT + RCV.WND
        //   RCV.NXT =< SEG.SEQ+SEG.LEN-1 < RCV.NXT + RCV.WND
        let seg_seqn = tcph.sequence_number();
        let mut seg_len = data.len() as u32;
        if tcph.fin() {
            seg_len += 1;
        }
        if tcph.syn() {
            seg_len += 1;
        }
        let wlen = self.recv.nxt.wrapping_add(self.recv.wnd as u32);

        if seg_len == 0 {
            if self.recv.wnd == 0 {
                if seg_seqn != self.recv.nxt {
                    return Ok(());
                }
            } else if !s_lte_x_lt_e_wrapping(self.recv.nxt, seg_seqn, wlen) {
                return Ok(());
            }
        } else {
            if self.recv.wnd == 0 {
                return Ok(());
            } else if !s_lte_x_lt_e_wrapping(self.recv.nxt, seg_seqn, wlen)
                && !s_lte_x_lt_e_wrapping(
                    self.recv.nxt,
                    seg_seqn.wrapping_add(seg_len).wrapping_sub(1),
                    wlen,
                )
            {
                return Ok(());
            }
        }

        match self.state {
            State::SyncRcvd => {
                // expect to get an ACK for our SYN
                if !tcph.ack() {
                    return Ok(());
                }

                self.state = State::Estab;
            }
            State::Estab => {
                unimplemented!();
            }
            _ => {}
        }
        Ok(())
    }
}

/// 🤯
pub fn s_lt_x_lte_e_wrapping(s: u32, x: u32, e: u32) -> bool {
    let (_d, _e) = ((x == e), (x != s));
    (_d && _e) || is_between_wrapped(s, x, e)
}

/// 🤯
pub fn s_lte_x_lt_e_wrapping(s: u32, x: u32, e: u32) -> bool {
    let (_d, _e) = ((x == s), (x != e));
    (_d && _e) || s_lt_x_lt_e_wrapping(s, x, e)
}

/// 🤯
pub fn s_lt_x_lt_e_wrapping(s: u32, x: u32, e: u32) -> bool {
    let (_a, _b, _c) = ((s < x), (x < e), (s < e));
    (_a && (_b == _c)) || (!_a && _b && !_c)
}

fn wrapping_lt(lhs: u32, rhs: u32) -> bool {
    // From RFC1323:
    //     TCP determines if a data segment is "old" or "new" by testing
    //     whether its sequence number is within 2**31 bytes of the left edge
    //     of the window, and if it is not, discarding the data as "old".  To
    //     insure that new data is never mistakenly considered old and vice-
    //     versa, the left edge of the sender's window has to be at most
    //     2**31 away from the right edge of the receiver's window.
    lhs.wrapping_sub(rhs) > (1 << 31)
}

fn is_between_wrapped(start: u32, x: u32, end: u32) -> bool {
    wrapping_lt(start, x) && wrapping_lt(x, end) && wrapping_lt(start, end)
}

#[cfg(test)]
mod tests {
    use crate::tcp::*;

    macro_rules! sltxltee_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (s, x, e, expected) = $value;
                assert_eq!(expected, s_lt_x_lte_e_wrapping(s, x, e));
            }
        )*
        }
    }

    sltxltee_tests! {
        case_eq_a: (0, 2, 2, true),
        case_eq_b: (u32::max_value(), 2, 2, true),
        case_eq_c: (u32::max_value().wrapping_add(1), 2, 2, true),
        all_eq: (u32::max_value(), u32::max_value(), u32::max_value(), false),
        no_wrap: (5, 18, 23, true),
        x_e_wrapped: (50, u32::max_value().wrapping_add(18), u32::max_value().wrapping_add(23), true),
        e_wrapped: (5, 18, u32::max_value().wrapping_add(2), true),
        start_wrapped: (3, 2, 1, false),
        no_bound_wrap_x_left: (2, 1, 3, false),
        no_bound_wrap_x_right: (1, 3, 2, false),
    }
}
