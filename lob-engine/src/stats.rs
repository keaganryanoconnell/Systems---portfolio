const HISTOGRAM_BUCKETS: usize = 1000;
const BUCKET_WIDTH_NS: u64 = 100;

pub struct LatencyStats {
    histogram: [u64; HISTOGRAM_BUCKETS],
    total_count: u64,
    total_sum_ns: u64,
    min_ns: u64,
    max_ns: u64,
}

impl LatencyStats {
    pub fn new() -> Self {
        Self {
            histogram: [0u64; HISTOGRAM_BUCKETS],
            total_count: 0,
            total_sum_ns: 0,
            min_ns: u64::MAX,
            max_ns: 0,
        }
    }
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyStats {
    pub fn record(&mut self, latency_ns: u64) {
        self.total_count += 1;
        self.total_sum_ns += latency_ns;

        if latency_ns < self.min_ns {
            self.min_ns = latency_ns;
        }
        if latency_ns > self.max_ns {
            self.max_ns = latency_ns;
        }

        let bucket = (latency_ns / BUCKET_WIDTH_NS).min(HISTOGRAM_BUCKETS as u64 - 1) as usize;
        self.histogram[bucket] += 1;
    }

    pub fn avg_ns(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }
        self.total_sum_ns as f64 / self.total_count as f64
    }

    pub fn percentile(&self, pct: f64) -> u64 {
        if self.total_count == 0 {
            return 0;
        }

        let target = (self.total_count as f64 * pct / 100.0).ceil() as u64;
        let mut cumulative: u64 = 0;

        for (i, &count) in self.histogram.iter().enumerate() {
            cumulative += count;
            if cumulative >= target {
                return (i as u64 + 1) * BUCKET_WIDTH_NS;
            }
        }

        HISTOGRAM_BUCKETS as u64 * BUCKET_WIDTH_NS
    }

    pub fn p50(&self) -> u64 {
        self.percentile(50.0)
    }
    pub fn p90(&self) -> u64 {
        self.percentile(90.0)
    }
    pub fn p99(&self) -> u64 {
        self.percentile(99.0)
    }
    pub fn p999(&self) -> u64 {
        self.percentile(99.9)
    }

    pub fn count(&self) -> u64 {
        self.total_count
    }
    pub fn min(&self) -> u64 {
        if self.total_count == 0 { 0 } else { self.min_ns }
    }
    pub fn max(&self) -> u64 {
        if self.total_count == 0 { 0 } else { self.max_ns }
    }

    pub fn print(&self, label: &str) {
        println!("=== {} ===", label);
        println!("  Count:    {}", self.total_count);
        println!("  Latency Distribution:");
        println!("    min:  {:>8} ns", self.min());
        println!("    avg:  {:>8.0} ns", self.avg_ns());
        println!("    p50:  {:>8} ns", self.p50());
        println!("    p90:  {:>8} ns", self.p90());
        println!("    p99:  {:>8} ns", self.p99());
        println!("    p999: {:>8} ns", self.p999());
        println!("    max:  {:>8} ns", self.max());
    }
}
