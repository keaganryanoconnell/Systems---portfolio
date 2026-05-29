use std::time::Instant;

use lob_engine::{LatencyStats, OrderBook, OrderPool, OrderRequest, OrderSide, RingBuffer};
use lob_engine::pool::MAX_ORDERS;

const ORDER_COUNT: usize = 1_000_000;

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

fn main() {
    println!("LOB Engine — Limit Order Book Matching Engine");
    println!("=============================================");
    println!();

    let mut pool = OrderPool::new();
    let mut book = OrderBook::new(&mut pool);
    let mut stats = LatencyStats::new();

    let ring = RingBuffer::<OrderRequest, 4096>::new();
    let pre_generated = generate_orders(ORDER_COUNT);

    println!("Pre-generated {} orders", pre_generated.len());
    println!("OrderPool capacity: {} slots", MAX_ORDERS);
    println!();
    println!("Processing...");

    let batch_size = 1000;
    let mut total_trades = 0u64;

    for chunk in pre_generated.chunks(batch_size) {
        for &order in chunk {
            while ring.push(order).is_err() {
                while let Some(req) = ring.pop() {
                    let t0 = Instant::now();
                    let trades = book.process_order(req);
                    let elapsed = t0.elapsed().as_nanos() as u64;
                    stats.record(elapsed);
                    total_trades += trades.len() as u64;
                }
            }
        }

        while let Some(req) = ring.pop() {
            let t0 = Instant::now();
            let trades = book.process_order(req);
            let elapsed = t0.elapsed().as_nanos() as u64;
            stats.record(elapsed);
            total_trades += trades.len() as u64;
        }
    }

    stats.print("LATENCY RESULTS (1,000,000 orders)");

    println!();
    let total_secs = stats.avg_ns() * stats.count() as f64 / 1_000_000_000.0;
    if total_secs > 0.0 {
        println!("Throughput: {:.0} orders/sec", stats.count() as f64 / total_secs);
    }
    println!("Total trades executed: {}", total_trades);
    println!(
        "Best bid: {:?}, Best ask: {:?}",
        book.best_bid(),
        book.best_ask()
    );
    println!("Last trade price: {}", book.last_trade_price);
}
