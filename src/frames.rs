use cobs::{decode, encode, max_encoding_length};
/// Module for frame generation and handling
use std::time::{Instant, Duration};

/// Maximum possible size of one frame (MTU=u16::MAX)
const MAX_FRAME_SIZE: usize = 65536;

/// Repeatable sequence to fill the frame data
const SEQUNCE: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

/// Handles incoming frames and checks frame content for validity
pub struct FrameHandler {
    counter: u32,
    statistics: FrameStatistics,
    buf: Box<[u8]>,
    speed_handler: SpeedMeasurer,
}

#[derive(Debug)]
pub struct FrameStatistics {
    pub session_id: u64,
    pub valid: u64,
    pub invalid: u64,
    pub out_of_order: u64,
    pub internally_bad: u64,
    pub lost: u64,
}

impl FrameStatistics {
    pub fn new(session_id: u64) -> Self {
        Self {
            session_id,
            valid: 0,
            invalid: 0,
            out_of_order: 0,
            internally_bad: 0,
            lost: 0,
        }
    }
}

impl FrameHandler {
    pub fn new() -> Self {
        Self {
            counter: u32::MAX,
            statistics: FrameStatistics::new(1),
            buf: vec![0; MAX_FRAME_SIZE].into_boxed_slice(),
            speed_handler: SpeedMeasurer::new(),
        }
    }
    pub fn reset(&mut self) {
        self.counter = u32::MAX;
        self.statistics = FrameStatistics::new(self.statistics.session_id + 1);
        self.speed_handler.reset();
    }
    /// Handle incoming frame
    ///
    /// Takes a null-terminated slice representing the whole frame
    pub fn handle(&mut self, frame: &[u8]) -> () {
        self.speed_handler.handle(frame.len());
        let frame = frame.strip_prefix(&[0]).unwrap_or(frame);
        let frame = if let Ok(decoded_len) = decode(frame, &mut self.buf) {
            &self.buf[..decoded_len]
        } else {
            self.statistics.invalid += 1;
            // eprintln!("Invalid because can't decode");
            return ();
        };
        self.counter = self.counter.wrapping_add(1);
        let Ok(counter) = TryInto::<[u8; 4]>::try_into(&frame[0..4]) else {
            self.statistics.invalid += 1;
            // println!("Invalid because can't read counter");
            return ();
        };
        let counter = u32::from_be_bytes(counter);
        match counter.cmp(&self.counter) {
            std::cmp::Ordering::Less => {
                // println!("Behind");
                self.statistics.out_of_order += 1;
                eprintln!("Received an out of order packet");
                self.counter = self.counter.wrapping_sub(1);
                self.statistics.lost = self.statistics.lost.saturating_sub(1);
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                // println!("Ahead");
                self.statistics.lost += Into::<u64>::into(counter - &self.counter);
                self.counter = counter;
            }
        }
        let data = &frame[4..];
        for i in data.chunks(SEQUNCE.len()) {
            if i.len() == SEQUNCE.len() {
                if i != SEQUNCE {
                    // println!("Improper chunk");
                    break;
                }
            } else {
                if &SEQUNCE[..i.len()] == i {
                    self.statistics.valid += 1;
                    return ();
                } else {
                    // println!("Improper end chunk");
                    break;
                }
            }
        }
        self.statistics.internally_bad += 1;
    }
    pub fn get_statistics(&self) -> Option<&FrameStatistics> {
        if self.counter == u32::MAX {
            return None
        }
        Some(&self.statistics)
    }
    pub fn get_speeds(&self) -> (u64, u64) {
        self.speed_handler.get_speeds()
    }

    pub fn get_latency(&self) -> u128 {
        self.speed_handler.get_latency()
    }
}

/// Frame Generator
///
/// every next() call returns a slice into the same buffer with modified sequence number
pub struct FrameBuilder {
    counter: u32,
    buf: Box<[u8]>,
    start_time: Instant,
    total_send: u64,
    cobs_encoded: Box<[u8]>,
}

impl FrameBuilder {
    /// Returns null-terminated slice presenting a cobs-encoded sequential frame
    pub fn next(&mut self) -> &[u8] {
        self.counter = self.counter.wrapping_add(1);
        let counter = &mut self.buf[0..4];
        counter.copy_from_slice(&self.counter.to_be_bytes());
        self.total_send += self.buf.len() as u64;
        let res = encode(&self.buf, &mut self.cobs_encoded);
        self.cobs_encoded[res] = 0;
        &self.cobs_encoded[..=res]
    }
    /// Geterates payload for frame builed
    pub fn new(mtu: u16) -> Self {
        const COBS_OVERHEAD: u16 = 2;
        let mut buf = vec![0_u8; Into::<usize>::into(mtu - COBS_OVERHEAD)].into_boxed_slice();
        let buf2 = vec![0_u8; max_encoding_length(buf.len())].into_boxed_slice();

        let data = &mut buf[4..];
        let mut seq_iter = SEQUNCE.iter().cycle();
        data.fill_with(|| *seq_iter.next().expect("Endless iterator"));
        Self {
            counter: u32::MAX,
            buf: buf,
            start_time: Instant::now(),
            total_send: 0,
            cobs_encoded: buf2,
        }
    }
    pub fn get_avg_kbps(&self) -> u64 {
        let dur = self.start_time.elapsed().as_secs();
        if dur == 0 {
            return 0;
        }
        (self.total_send / 1024) * 8 / dur
    }
}

pub struct SpeedMeasurer {
    session_start: Option<Instant>,
    session_received: usize,
    measure_start: Option<Instant>,
    measure_received: usize,
    measure_speed: u64,
    measure_latencies: Vec<u128>,
    prev_recv: Option<Instant>,
}

impl SpeedMeasurer {
    pub fn new() -> Self {
        Self {
            session_start: None,
            session_received: 1,
            measure_start: None,
            measure_received: 1,
            measure_speed: 0,
            measure_latencies: vec![],
            prev_recv: None,
        }
    }

    pub fn handle(&mut self, len: usize) {
        let time = Instant::now();
        let _session_start = self.session_start.get_or_insert(time);
        self.session_received += len;
        let measure_start = self.measure_start.get_or_insert(time);
        if let Some(prev) = &mut self.prev_recv {
            let latency = time.duration_since(*prev);
            *prev = time;
            if latency.as_millis() > 100 { // First in Burst
                self.measure_latencies.clear();
            } else {
                self.measure_latencies.push(latency.as_micros());
            }
            // println!("latency for this packet = {}", latency.as_micros())
        } else {
            self.prev_recv = Some(time);
        }
        self.measure_received += len;
        if measure_start.elapsed() >= Duration::from_millis(1000) {
            self.measure_speed = (self.measure_received as u128 * 8 / 1024 * 1000 / measure_start.elapsed().as_millis()).try_into().unwrap();
            *measure_start = Instant::now();
            self.measure_received = 1;
        };
    }
    pub fn get_speeds(&self) -> (u64, u64) {
        if self.session_start.is_none() {return (0, 0)}
        let avg_session_speed = self.session_received as u128 * 8 / 1024 * 1000 / self.session_start.unwrap().elapsed().as_millis();
        return (avg_session_speed.try_into().unwrap_or(0), self.measure_speed)
    }
    pub fn reset(&mut self) {
        self.session_start = None;
        self.measure_start = None;
        self.session_received = 1;
        self.measure_received = 1;
        self.measure_speed = 0;
    }
    pub fn get_latency(&self) -> u128 {
        if self.measure_latencies.len() == 0 {
            return 0
        }
        let sum: u128 = self.measure_latencies.iter().sum();
        // / self.measure_latencies.len()
        sum / self.measure_latencies.len() as u128
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Takes a long time
    /// It's assumed in the code above, that COBS overhead will always be 2 for
    /// current SEQENCE, if the sequence changes, COBS might give a bigger overhead.
    fn test_cobs_overhead() {
        let mut builder = FrameBuilder::new(1500);
        for _ in (0..=u32::MAX).step_by(1) {
            assert_eq!(builder.next().len(), 1500)
        }
    }

    #[test]
    fn test_frame_by_frame() {
        let mut builder = FrameBuilder::new(1500);
        let mut handler = FrameHandler::new();
        for i in 1..=5 {
            let frame = builder.next();
            handler.handle(frame);
            assert_eq!(handler.get_statistics().unwrap().valid, i)
        }
    }
}
