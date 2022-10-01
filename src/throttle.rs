use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

pub struct RequestThrottle {
    enabled: bool,
    last_request_timestamp: SystemTime,
    requests_per_second_limit: usize,
    requests_per_second: usize,
}

impl RequestThrottle {
    pub fn new(requests_per_second_limit: usize) -> RequestThrottle {
        if requests_per_second_limit > 0 {
            RequestThrottle {
                enabled: true,
                last_request_timestamp: SystemTime::now(),
                requests_per_second_limit,
                requests_per_second: 0,
            }
        } else {
            RequestThrottle {
                enabled: false,
                last_request_timestamp: SystemTime::now(),
                requests_per_second_limit,
                requests_per_second: 0,
            }
        }
    }

    pub fn increment_or_sleep(&mut self, inc: usize) {
        if self.enabled
            && SystemTime::now()
                .duration_since(self.last_request_timestamp)
                .unwrap()
                .as_secs()
                < 1
        {
            if self.requests_per_second == self.requests_per_second_limit {
                sleep(Duration::from_secs(1));
                self.requests_per_second = 0;
                self.last_request_timestamp = SystemTime::now();
            } else {
                self.requests_per_second += inc;
                self.last_request_timestamp = SystemTime::now();
            }
        }
    }
}
