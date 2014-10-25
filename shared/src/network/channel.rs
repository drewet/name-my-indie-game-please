use std;
use std::collections::{Deque, RingBuf};

use std::io::net::udp::UdpStream;
use std::io::IoResult;
//use std::io::net::ip::{Ipv4Addr};

type SequenceNr = u32;

fn overflow_aware_compare(a: SequenceNr, b: SequenceNr) -> std::cmp::Ordering {
    use std::cmp::{max, min};
    
    let abs_difference = max(a, b) - min(a, b);
    
    if abs_difference < std::u32::MAX / 2 {
        a.cmp(&b)
    } else {
        b.cmp(&a)
    }
}

pub struct NetChannel {
    last_outgoing: SequenceNr,
    last_acked_outgoing: SequenceNr,
    last_incoming: SequenceNr,
    
    send_times: RingBuf<f64>,
    latency: f64,

    stream: UdpStream,
}

impl NetChannel {

    /// Doesn't actually do any networking.
    pub fn from_stream(mut stream: UdpStream) -> NetChannel {
        stream.as_socket(|sock| sock.set_read_timeout(Some(0)));

        NetChannel {
            last_outgoing: 0,
            last_acked_outgoing: 0,
            last_incoming: 0,

            send_times: RingBuf::new(),
            latency: 0.,

            stream: stream,
        }
    }

    pub fn send_unreliable(&mut self, data: &[u8]) -> IoResult<()> {
        let mut buf = std::io::MemWriter::with_capacity(data.len() + 16);
        
        self.last_outgoing += 1;

        self.send_times.push(::time::precise_time_s());

        try!(buf.write_le_u32(self.last_outgoing));
        try!(buf.write_le_u32(self.last_incoming));
        try!(buf.write_le_u64(data.len() as u64));
        try!(buf.write(data));

        self.stream.write(buf.get_ref())
    }

    pub fn try_recv_unreliable(&mut self) -> IoResult<Vec<u8>> {
        let mut buf = [0u8, ..8192];
        
        let mut len = 0;

        while len == 0 {
            len = try!(self.stream.read(&mut buf));
        }

        let mut buf = std::io::BufReader::new(buf.slice_to(len));
        
        let sequence_number = try!(buf.read_le_u32());
        let acked_sequence_number = try!(buf.read_le_u32());
        let payload_len = try!(buf.read_le_u64());
        let payload = try!(buf.read_exact(payload_len as uint));

        self.last_incoming = sequence_number;
        self.ack(acked_sequence_number);

        Ok(payload)
    }

    pub fn recv_unreliable(&mut self) -> IoResult<Vec<u8>> {
        loop {
            match self.try_recv_unreliable() {
                Ok(result) => return Ok(result),
                Err(ref e) if e.kind == std::io::TimedOut => continue,
                Err(e) => return Err(e)
            }
        }
    }

    fn ack(&mut self, seq: SequenceNr) {
        let prev_acked = self.last_acked_outgoing;
        self.last_acked_outgoing = seq;

        let curtime = ::time::precise_time_s();

        for _ in range(0, seq - prev_acked) {
            let sendtime = self.send_times.pop_front().expect("Too many acks!");
            self.latency = (curtime - sendtime);
        }
    }

    pub fn get_latency(&self) -> f64 {
        self.latency
    }
}

mod test {
    use std::io::net::udp::UdpSocket;
    use std::io::net::ip::{Ipv4Addr, SocketAddr};
    use super::NetChannel;

    fn get_channel_pair() -> (NetChannel, NetChannel) {
        let mut sock1 = UdpSocket::bind(SocketAddr{ ip: Ipv4Addr(127, 0, 0, 1), port: 0 }).unwrap();
        let mut sock2 = UdpSocket::bind(SocketAddr{ ip: Ipv4Addr(127, 0, 0, 1), port: 0 }).unwrap();
        let sock1_addr = sock1.socket_name().unwrap();

        let stream1 = sock1.connect(sock2.socket_name().unwrap());
        let stream2 = sock2.connect(sock1_addr);

        (NetChannel::from_stream(stream1), NetChannel::from_stream(stream2))
    }


    #[test]
    fn smoke_netchannel() {
        let (mut chan1, mut chan2) = get_channel_pair();

        chan1.send_unreliable(b"Hello, world!").unwrap();
        let result = chan2.recv_unreliable().unwrap();
        assert_eq!(result.as_slice(), b"Hello, world!");
    }

    #[test]
    fn double_ended() {
        let (mut chan1, mut chan2) = get_channel_pair();

        chan1.send_unreliable(b"Hello, chan2!").unwrap();
        let result = chan2.recv_unreliable().unwrap();
        assert_eq!(result.as_slice(), b"Hello, chan2!");

        chan2.send_unreliable(b"Hello, chan1!").unwrap();
        let result = chan1.recv_unreliable().unwrap();
        assert_eq!(result.as_slice(), b"Hello, chan1!");
    }
}
