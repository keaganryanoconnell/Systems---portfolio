use crate::pool::OrderPool;
use crate::types::{OrderRequest, OrderSide, OrderStatus, Trade};

const MAX_PRICE_LEVELS: usize = 512;
const ORDERS_PER_LEVEL: usize = 48;

pub struct PriceLevel {
    price: u64,
    order_ids: [u32; ORDERS_PER_LEVEL],
    order_count: u8,
}

impl PriceLevel {
    fn new(price: u64) -> Self {
        Self {
            price,
            order_ids: [0u32; ORDERS_PER_LEVEL],
            order_count: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.order_count as usize >= ORDERS_PER_LEVEL
    }

    fn push(&mut self, order_id: u32) {
        let idx = self.order_count as usize;
        self.order_ids[idx] = order_id;
        self.order_count += 1;
    }

    fn remove(&mut self, order_id: u32) {
        for i in 0..self.order_count as usize {
            if self.order_ids[i] == order_id {
                self.order_ids[i] = self.order_ids[self.order_count as usize - 1];
                self.order_count -= 1;
                return;
            }
        }
    }

    fn first_order_id(&self) -> u32 {
        self.order_ids[0]
    }
}

pub struct OrderBook<'a> {
    pool: &'a mut OrderPool,
    bid_levels: [PriceLevel; MAX_PRICE_LEVELS],
    bid_count: usize,
    ask_levels: [PriceLevel; MAX_PRICE_LEVELS],
    ask_count: usize,
    pub last_trade_price: u64,
}

impl<'a> OrderBook<'a> {
    pub fn new(pool: &'a mut OrderPool) -> Self {
        let book = Self {
            pool,
            bid_levels: std::array::from_fn(|_| PriceLevel::new(0)),
            bid_count: 0,
            ask_levels: std::array::from_fn(|_| PriceLevel::new(0)),
            ask_count: 0,
            last_trade_price: 0,
        };
        book
    }

    pub fn process_order(&mut self, req: OrderRequest) -> Vec<Trade> {
        match req.side {
            OrderSide::Buy => self.process_buy(req),
            OrderSide::Sell => self.process_sell(req),
        }
    }

    fn process_buy(&mut self, req: OrderRequest) -> Vec<Trade> {
        let mut trades = Vec::with_capacity(16);
        let mut remaining = req.quantity;

        while remaining > 0 && self.ask_count > 0 {
            let best_ask_price = self.ask_levels[0].price;
            if best_ask_price > req.price {
                break;
            }

            let mut filled_here = false;
            let level = &mut self.ask_levels[0];

            while level.order_count > 0 && remaining > 0 {
                let order_id = level.first_order_id();
                let resting_qty = {
                    let order = self.pool.get(order_id);
                    order.remaining
                };

                let fill_qty = remaining.min(resting_qty);
                let fill_price = level.price;

                remaining -= fill_qty;

                {
                    let order = self.pool.get_mut(order_id);
                    order.remaining -= fill_qty;
                    if order.remaining == 0 {
                        order.status = OrderStatus::Filled;
                    } else {
                        order.status = OrderStatus::PartiallyFilled;
                    }
                }

                let resting_side = self.pool.get(order_id).side;
                let (buy_id, sell_id) = if resting_side == OrderSide::Sell {
                    (u32::MAX, order_id)
                } else {
                    (order_id, u32::MAX)
                };

                trades.push(Trade {
                    buy_order_id: buy_id,
                    sell_order_id: sell_id,
                    price: fill_price,
                    quantity: fill_qty,
                });

                self.last_trade_price = fill_price;

                if self.pool.get(order_id).remaining == 0 {
                    level.remove(order_id);
                    self.pool.free(order_id);
                }
            }

            if level.order_count == 0 {
                self.remove_ask_level(0);
                filled_here = true;
            }

            if !filled_here {
                break;
            }
        }

        if trades.is_empty() {
            if remaining > 0 {
                let order_id = self.pool.alloc(OrderSide::Buy, req.price, remaining);
                if let Some(id) = order_id {
                    self.insert_bid(req.price, id);
                }
            }
        } else if remaining > 0 {
            let order_id = self.pool.alloc(OrderSide::Buy, req.price, remaining);
            if let Some(id) = order_id {
                self.insert_bid(req.price, id);
            }
        }

        trades
    }

    fn process_sell(&mut self, req: OrderRequest) -> Vec<Trade> {
        let mut trades = Vec::with_capacity(16);
        let mut remaining = req.quantity;

        while remaining > 0 && self.bid_count > 0 {
            let best_bid_price = self.bid_levels[0].price;
            if best_bid_price < req.price {
                break;
            }

            let level = &mut self.bid_levels[0];

            while level.order_count > 0 && remaining > 0 {
                let order_id = level.first_order_id();
                let resting_qty = {
                    let order = self.pool.get(order_id);
                    order.remaining
                };

                let fill_qty = remaining.min(resting_qty);
                let fill_price = level.price;

                remaining -= fill_qty;

                {
                    let order = self.pool.get_mut(order_id);
                    order.remaining -= fill_qty;
                    if order.remaining == 0 {
                        order.status = OrderStatus::Filled;
                    } else {
                        order.status = OrderStatus::PartiallyFilled;
                    }
                }

                trades.push(Trade {
                    buy_order_id: order_id,
                    sell_order_id: u32::MAX,
                    price: fill_price,
                    quantity: fill_qty,
                });

                self.last_trade_price = fill_price;

                if self.pool.get(order_id).remaining == 0 {
                    level.remove(order_id);
                    self.pool.free(order_id);
                }
            }

            if level.order_count == 0 {
                self.remove_bid_level(0);
            }
        }

        if trades.is_empty() && remaining > 0 {
            if let Some(id) = self.pool.alloc(OrderSide::Sell, req.price, remaining) {
                self.insert_ask(req.price, id);
            }
        } else if remaining > 0 {
            if let Some(id) = self.pool.alloc(OrderSide::Sell, req.price, remaining) {
                self.insert_ask(req.price, id);
            }
        }

        trades
    }

    fn insert_bid(&mut self, price: u64, order_id: u32) {
        if self.bid_count >= MAX_PRICE_LEVELS {
            return;
        }
        let pos = self.find_bid_insert_pos(price);
        if pos < self.bid_count && self.bid_levels[pos].price == price && !self.bid_levels[pos].is_full() {
            self.bid_levels[pos].push(order_id);
        } else {
            self.shift_bids_right(pos);
            let mut level = PriceLevel::new(price);
            level.push(order_id);
            self.bid_levels[pos] = level;
            self.bid_count += 1;
        }
    }

    fn insert_ask(&mut self, price: u64, order_id: u32) {
        if self.ask_count >= MAX_PRICE_LEVELS {
            return;
        }
        let pos = self.find_ask_insert_pos(price);
        if pos < self.ask_count && self.ask_levels[pos].price == price && !self.ask_levels[pos].is_full() {
            self.ask_levels[pos].push(order_id);
        } else {
            self.shift_asks_right(pos);
            let mut level = PriceLevel::new(price);
            level.push(order_id);
            self.ask_levels[pos] = level;
            self.ask_count += 1;
        }
    }

    fn find_bid_insert_pos(&self, price: u64) -> usize {
        let mut lo = 0;
        let mut hi = self.bid_count;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.bid_levels[mid].price <= price {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    }

    fn find_ask_insert_pos(&self, price: u64) -> usize {
        let mut lo = 0;
        let mut hi = self.ask_count;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.ask_levels[mid].price >= price {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    }

    fn shift_bids_right(&mut self, from: usize) {
        for i in (from..self.bid_count).rev() {
            let src = std::ptr::from_ref(&self.bid_levels[i]);
            let dst = std::ptr::from_mut(&mut self.bid_levels[i + 1]);
            unsafe { std::ptr::copy_nonoverlapping(src, dst, 1) };
        }
    }

    fn shift_asks_right(&mut self, from: usize) {
        for i in (from..self.ask_count).rev() {
            let src = std::ptr::from_ref(&self.ask_levels[i]);
            let dst = std::ptr::from_mut(&mut self.ask_levels[i + 1]);
            unsafe { std::ptr::copy_nonoverlapping(src, dst, 1) };
        }
    }

    fn remove_bid_level(&mut self, idx: usize) {
        let src = std::ptr::from_ref(&self.bid_levels[idx + 1]);
        let dst = std::ptr::from_mut(&mut self.bid_levels[idx]);
        let count = self.bid_count - idx - 1;
        if count > 0 {
            unsafe { std::ptr::copy(src, dst, count) };
        }
        self.bid_count -= 1;
    }

    fn remove_ask_level(&mut self, idx: usize) {
        let src = std::ptr::from_ref(&self.ask_levels[idx + 1]);
        let dst = std::ptr::from_mut(&mut self.ask_levels[idx]);
        let count = self.ask_count - idx - 1;
        if count > 0 {
            unsafe { std::ptr::copy(src, dst, count) };
        }
        self.ask_count -= 1;
    }

    pub fn best_bid(&self) -> Option<u64> {
        if self.bid_count > 0 {
            Some(self.bid_levels[0].price)
        } else {
            None
        }
    }

    pub fn best_ask(&self) -> Option<u64> {
        if self.ask_count > 0 {
            Some(self.ask_levels[0].price)
        } else {
            None
        }
    }

    pub fn bid_levels(&self) -> &[PriceLevel] {
        &self.bid_levels[..self.bid_count]
    }

    pub fn ask_levels(&self) -> &[PriceLevel] {
        &self.ask_levels[..self.ask_count]
    }
}
