use std;
use std::collections::{Deque, RingBuf};
use std::io::IoResult;

pub type SequenceNr = u32;

pub fn overflow_aware_compare(a: SequenceNr, b: SequenceNr) -> std::cmp::Ordering {
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
}

impl NetChannel {

    pub fn new() -> NetChannel {
        NetChannel {
            last_outgoing: 0,
            last_acked_outgoing: 0,
            last_incoming: 0,

            send_times: RingBuf::new(),
            latency: 0.,
        }
    }

    pub fn get_outgoing_sequencenr(&self) -> SequenceNr { self.last_outgoing }
    pub fn get_acked_outgoing_sequencenr(&self) -> SequenceNr { self.last_acked_outgoing }

    pub fn send_unreliable(&mut self, data: &[u8]) -> IoResult<Vec<u8>> {
        let mut buf = std::io::MemWriter::with_capacity(data.len() + 16);
        
        self.last_outgoing += 1;

        self.send_times.push(::time::precise_time_s());

        try!(buf.write_le_u32(self.last_outgoing));
        try!(buf.write_le_u32(self.last_incoming));
        try!(buf.write_le_u64(data.len() as u64));
        try!(buf.write(data));

        Ok(buf.unwrap())
    }

    pub fn recv_unreliable(&mut self, datagram: &[u8]) -> IoResult<Vec<u8>> {
        let mut buf = std::io::BufReader::new(datagram);
        
        let sequence_number = try!(buf.read_le_u32());
        let acked_sequence_number = try!(buf.read_le_u32());
        let payload_len = try!(buf.read_le_u64());
        let payload = try!(buf.read_exact(payload_len as uint));

        self.last_incoming = sequence_number;
        self.ack(acked_sequence_number);

        Ok(payload)
    }

    fn ack(&mut self, seq: SequenceNr) {
        let prev_acked = self.last_acked_outgoing;
        self.last_acked_outgoing = seq;

        let curtime = ::time::precise_time_s();

        for _ in range(0, seq - prev_acked) {
            let sendtime = self.send_times.pop_front().expect("Too many acks!");
            self.latency = curtime - sendtime;
        }
    }

    pub fn get_latency(&self) -> f64 {
        self.latency
    }

}

#[cfg(test)]
mod test {
    use super::NetChannel;

    #[test]
    fn smoke_netchannel() {
        let (mut chan1, mut chan2) = (NetChannel::new(), NetChannel::new());

        let result = chan2.recv_unreliable(chan1.send_unreliable(b"Hello, world!").unwrap().as_slice()).unwrap();
        assert_eq!(result.as_slice(), b"Hello, world!");
    }

    #[test]
    fn double_ended() {
        let (mut chan1, mut chan2) = (NetChannel::new(), NetChannel::new());

        let pkt = chan1.send_unreliable(b"Hello, chan2!").unwrap();
        let result = chan2.recv_unreliable(pkt.as_slice()).unwrap();
        assert_eq!(result.as_slice(), b"Hello, chan2!");

        let pkt = chan2.send_unreliable(b"Hello, chan1!").unwrap();
        let result = chan1.recv_unreliable(pkt.as_slice()).unwrap();
        assert_eq!(result.as_slice(), b"Hello, chan1!");
    }
}
