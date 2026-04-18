use std::fmt;

pub fn analyze(flag: bool, values: &[i32]) -> i32 {
    // hot path
    if flag {
        return values.len() as i32;
    }

    let mut total = 0;
    for value in values {
        if *value > 0 {
            total += value;
        }
    }
    total
}

struct Worker;

impl Worker {
    pub fn compute(&self, limit: usize) -> usize {
        let mut count = 0;
        while count < limit {
            count += 1;
        }
        count
    }
}
