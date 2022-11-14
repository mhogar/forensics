use core::fmt;

pub struct Partition {
    partition_type: String,
    start_lba: u64,
    end_lba: u64,
}

impl Partition {
    pub fn new1(partition_type: String, start_lba: u64, end_lba: u64) -> Partition {
        Partition { partition_type, start_lba, end_lba }
    }

    pub fn new2(partition_type: String, start_lba: u64, num_sectors: u64) -> Partition {
        Partition::new1(partition_type, start_lba, (start_lba + num_sectors).saturating_sub(1))
    }

    pub fn num_sectors(&self) -> u64 {
        self.end_lba - self.start_lba
    }

    pub fn size_mb(&self) -> f32 {
        (self.num_sectors() as f32) / 2048.0
    }
}

impl fmt::Display for Partition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t{}\t{}\t{}", self.partition_type, self.start_lba, self.start_lba + self.num_sectors(), self.size_mb())
    }
}
