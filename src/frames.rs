use cobs::{decode, encode, max_encoding_length};
/// Module for frame generation and handling
use std::time::Instant;

/// Maximum possible size of one frame (MTU=u16::MAX)
const MAX_FRAME_SIZE: usize = 65536;

/// Repeatable sequence to fill the frame data
const SEQUNCE: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

/// Handles incoming frames and checks frame content for validity
pub struct FrameHandler {
    counter: u32,
    statistics: FrameStatistics,
    start_time: Option<Instant>,
    total_received: u64,
    buf: Box<[u8]>,
}

#[derive(Default, Debug)]
pub struct FrameStatistics {
    pub valid: u64,
    pub invalid: u64,
    pub out_of_order: u64,
    pub internally_bad: u64,
    pub lost: u64,
}

impl FrameHandler {
    pub fn new() -> Self {
        Self {
            counter: u32::MAX,
            statistics: Default::default(),
            start_time: None,
            total_received: 0,
            buf: vec![0; MAX_FRAME_SIZE].into_boxed_slice(),
        }
    }
    pub fn reset(&mut self) {
        self.counter = u32::MAX;
        self.statistics = Default::default();
        self.start_time = None;
    }
    pub fn handle(&mut self, frame: &[u8]) -> () {
        if let None = self.start_time {
            self.start_time = Some(Instant::now());
        }
        let frame = &frame[..frame.len() - 1];
        let frame = if let Ok(size) = decode(frame, &mut self.buf) {
            &self.buf[..size]
        } else {
            self.statistics.invalid += 1;
            println!("Invalid because can't decode");
            return ();
        };
        self.counter = self.counter.wrapping_add(1);
        let Ok(counter) = TryInto::<[u8; 4]>::try_into(&frame[0..4]) else {
            self.statistics.invalid += 1;
            println!("Invalid because can't read counter");
            return ()
        };
        let Ok(length) = TryInto::<[u8; 4]>::try_into(&frame[4..8]) else {
            self.statistics.invalid += 1;
            println!("Invalid because can't read length");
            return ()
        };
        let counter = u32::from_be_bytes(counter);
        let length = u32::from_be_bytes(length);
        if length as usize > frame.len() {
            self.statistics.invalid += 1;
            println!("Invalid because length is wrong: {}", frame.len());
            return ();
        };
        match counter.cmp(&self.counter) {
            std::cmp::Ordering::Less => {
                self.statistics.out_of_order += 1;
                self.counter = self.counter.wrapping_sub(1);
                self.statistics.lost = self.statistics.lost.saturating_sub(1);
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                // println!("Ahead")
                self.statistics.lost += Into::<u64>::into(counter - &self.counter);
                self.counter = counter;
            }
        }
        let data = &frame[8..];
        for i in data.chunks(SEQUNCE.len()) {
            if i.len() == SEQUNCE.len() {
                if i != SEQUNCE {
                    // println!("Improper chunk");
                    break;
                }
            } else {
                if &SEQUNCE[..i.len()] == i {
                    self.statistics.valid += 1;
                    self.total_received += Into::<u64>::into(length);
                    return ();
                } else {
                    // println!("Improper end chunk");
                    break;
                }
            }
        }
        self.statistics.internally_bad += 1;
    }
    pub fn get_statistics(&self) -> &FrameStatistics {
        &self.statistics
    }
    pub fn get_avg_kbps(&self) -> u64 {
        if let Some(start_time) = self.start_time {
            let dur = start_time.elapsed().as_secs();
            if dur == 0 {
                return 0;
            }
            (self.total_received / 1024) * 8 / dur
        } else {
            0
        }
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
    pub fn next(&mut self) -> &[u8] {
        self.counter = self.counter.wrapping_add(1);
        let counter = &mut self.buf[0..4];
        counter.copy_from_slice(&self.counter.to_be_bytes());
        self.total_send += self.buf.len() as u64;
        let res = encode(&self.buf, &mut self.cobs_encoded);
        self.cobs_encoded[res] = 0;
        // &self.buf
        &self.cobs_encoded[..=res]
    }
    pub fn new(mtu: u16) -> Self {
        const COBS_OVERHEAD: u16 = 2;
        let mut buf = vec![0_u8; Into::<usize>::into(mtu - COBS_OVERHEAD)].into_boxed_slice();
        let buf2 = vec![0_u8; max_encoding_length(buf.len())].into_boxed_slice();
        let l = buf.len() as u32;
        let length = &mut buf[4..8];
        length.copy_from_slice(&l.to_be_bytes());
        let data = &mut buf[8..];
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
