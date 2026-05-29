use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Instant;

use lob_engine::{LatencyStats, OrderBook, OrderPool, OrderRequest, OrderSide, RingBuffer};

fn generate_orders(count: usize) -> Vec<OrderRequest> {
    let mut orders = Vec::with_capacity(count);
    let base_price = 100_000u64;

    for i in 0..count {
        let side = if i % 3 == 0 {
            OrderSide::Sell
        } else {
            OrderSide::Buy
        };
        let offset = (i as u64 % 200) as i64 - 100;
        let price = if offset >= 0 {
            base_price + offset as u64
        } else {
            base_price.saturating_sub((-offset) as u64)
        };
        let qty = ((i % 10) + 1) * 100;
        orders.push(OrderRequest {
            side,
            price: price.max(1),
            quantity: qty as u32,
        });
    }
    orders
}

fn bench_matching_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("lob_matching");
    group.sample_size(10);

    group.bench_function("process_1_000_000_orders", |b| {
        let orders = generate_orders(1_000_000);

        b.iter(|| {
            let mut pool = OrderPool::new();
            let mut book = OrderBook::new(&mut pool);
            let mut stats = LatencyStats::new();

            for &order in &orders {
                let t0 = Instant::now();
                let _trades = book.process_order(black_box(order));
                let elapsed = t0.elapsed().as_nanos() as u64;
                stats.record(elapsed);
            }

            black_box(stats.p99());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_matching_throughput);
criterion_main!(benches);
