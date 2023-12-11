use std::time::Instant;

const SEQUNCE: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

// const TEST_SEQUNCE: [u8; 16] = [
//     b'l', b'm', b'a', b'o', b'l', b'm', b'a', b'o', b'l', b'm', b'a', b'o', b'l', b'm', b'a', b'o',
// ];

pub struct FrameHandler {
    counter: u32,
    statistics: FrameStatistics,
    start_time: Option<Instant>,
    total_received: u64,
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
        (self.counter, _) = self.counter.overflowing_add(1);
        let Ok(counter) = TryInto::<[u8; 4]>::try_into(&frame[0..4]) else {
            self.statistics.invalid += 1;
            return ()
        };
        let Ok(length) = TryInto::<[u8; 4]>::try_into(&frame[4..8]) else {
            self.statistics.invalid += 1;
            return ()
        };
        let counter = u32::from_be_bytes(counter);
        let length = u32::from_be_bytes(length);
        if length as usize != frame.len() {
            self.statistics.invalid += 1;
            return ();
        };
        match counter.cmp(&self.counter) {
            std::cmp::Ordering::Less => {
                self.statistics.out_of_order += 1;
                (self.counter, _) = self.counter.overflowing_sub(1);
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

pub struct FrameBuilder {
    counter: u32,
    buf: Box<[u8]>,
    start_time: Instant,
    total_send: u64,
}

impl FrameBuilder {
    pub fn next(&mut self) -> &[u8] {
        (self.counter, _) = self.counter.overflowing_add(1);
        let counter = &mut self.buf[0..4];
        counter.copy_from_slice(&self.counter.to_be_bytes());
        self.total_send += self.buf.len() as u64;
        &self.buf
    }
    pub fn new(mtu: u16) -> Self {
        let mut buf = vec![0_u8; Into::<usize>::into(mtu)].into_boxed_slice();
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
            total_send: 0
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
