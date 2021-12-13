use crate::header_chain::StoredBlockHeader;

pub type Error = String;

pub type ChainDBResult<T> = Result<T, Error>;

pub trait ReadonlyChainDB {
    fn header_at(&mut self, height: u32) -> ChainDBResult<Option<StoredBlockHeader>>;
    fn tip_height(&mut self) -> ChainDBResult<Option<u32>>;

    // gets last 12 timestamps.
    // if there is less than 12 blocks in store, return the existing timestamps only.
    fn last_timestamps_at(&mut self, height: u32) -> ChainDBResult<Timestamps> {
        let start = height.saturating_sub(11);
        let mut timestamps = Timestamps(Vec::with_capacity((height - start - 1) as usize));
        for h in (start..height).rev() {
            match self.header_at(h)? {
                Some(header) => timestamps.push(header.header.time),
                None => break,
            }
        }
        Ok(timestamps)
    }

    //https://blog.bitmex.com/bitcoins-block-timestamp-protection-rules/
    fn mpt_at(&mut self, height: u32) -> ChainDBResult<Option<u32>> {
        Ok(self.last_timestamps_at(height)?.mid())
    }
}

#[derive(Default)]
pub struct Timestamps(pub Vec<u32>);

impl Timestamps {
    pub fn mid(&self) -> Option<u32> {
        let mut vec = self.0.clone();
        vec.sort();
        let len = vec.len();
        if len == 0 {
            None
        } else if len == 1 {
            Some(vec[0])
        } else if len % 2 == 0 {
            let mid = len / 2;
            Some((vec[mid - 1] + vec[mid]) / 2)
        } else {
            Some(vec[len / 2])
        }
    }
    pub fn push(&mut self, timestamp: u32) {
        if self.0.len() == 11 {
            self.0.remove(0);
        }
        self.0.push(timestamp)
    }
}

pub trait ChainDB: ReadonlyChainDB {
    fn store_header(&mut self, height: u32, block_header: StoredBlockHeader) -> ChainDBResult<()>;
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_timestamp_mid() {
        let timestamps = Timestamps(vec![]);
        assert!(timestamps.mid().is_none());
        let timestamps = Timestamps(vec![10]);
        assert_eq!(timestamps.mid().unwrap(), 10);
        let timestamps = Timestamps(vec![10, 20]);
        assert_eq!(timestamps.mid().unwrap(), 15);
        let timestamps = Timestamps(vec![10, 21]);
        assert_eq!(timestamps.mid().unwrap(), 15);
        let timestamps = Timestamps(vec![10, 20, 100]);
        assert_eq!(timestamps.mid().unwrap(), 20);
        let timestamps = Timestamps(vec![10, 100, 20]);
        assert_eq!(timestamps.mid().unwrap(), 20);
        let timestamps = Timestamps(vec![10, 100, 20, 4000]);
        assert_eq!(timestamps.mid().unwrap(), 60);
        let timestamps = Timestamps(vec![10, 100, 100, 4000]);
        assert_eq!(timestamps.mid().unwrap(), 100);
        let timestamps = Timestamps(vec![10, 100, 100, 4000, 100]);
        assert_eq!(timestamps.mid().unwrap(), 100);
        assert_eq!(timestamps.0, vec![10, 100, 100, 4000, 100]);
    }

    #[test]
    fn test_timestamp_push() {
        let mut timestamps = Timestamps(vec![0, 1, 3, 5, 7, 9, 0, 2, 4, 6]);
        timestamps.push(10);
        assert_eq!(timestamps.0, vec![0, 1, 3, 5, 7, 9, 0, 2, 4, 6, 10]);
        timestamps.push(2);
        assert_eq!(timestamps.0, vec![1, 3, 5, 7, 9, 0, 2, 4, 6, 10, 2]);
    }
}
