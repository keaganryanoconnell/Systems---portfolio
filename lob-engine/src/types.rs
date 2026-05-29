#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub id: u32,
    pub side: OrderSide,
    pub price: u64,
    pub quantity: u32,
    pub remaining: u32,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Copy)]
pub struct OrderRequest {
    pub side: OrderSide,
    pub price: u64,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Trade {
    pub buy_order_id: u32,
    pub sell_order_id: u32,
    pub price: u64,
    pub quantity: u32,
}
