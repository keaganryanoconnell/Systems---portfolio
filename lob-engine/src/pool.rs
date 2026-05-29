use std::mem::MaybeUninit;

use crate::types::{Order, OrderSide, OrderStatus};

pub const MAX_ORDERS: usize = 1_000_000;

pub struct OrderPool {
    entries: Box<[MaybeUninit<Order>]>,
    free_stack: Box<[u32]>,
    free_head: u32,
    next_id: u32,
}

impl OrderPool {
    pub fn new() -> Self {
        let pool = Self {
            entries: (0..MAX_ORDERS)
                .map(|_| MaybeUninit::uninit())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            free_stack: vec![0u32; MAX_ORDERS].into_boxed_slice(),
            free_head: 0,
            next_id: 0,
        };
        pool
    }

    pub fn alloc(&mut self, side: OrderSide, price: u64, quantity: u32) -> Option<u32> {
        if self.next_id as usize >= MAX_ORDERS {
            if self.free_head == 0 {
                return None;
            }
            self.free_head -= 1;
            let idx = self.free_stack[self.free_head as usize];
            let order = Order {
                id: idx,
                side,
                price,
                quantity,
                remaining: quantity,
                status: OrderStatus::New,
            };
            self.entries[idx as usize].write(order);
            return Some(idx);
        }

        let idx = self.next_id;
        self.next_id += 1;

        let order = Order {
            id: idx,
            side,
            price,
            quantity,
            remaining: quantity,
            status: OrderStatus::New,
        };
        self.entries[idx as usize].write(order);
        Some(idx)
    }

    pub fn free(&mut self, id: u32) {
        let idx = id as usize;
        unsafe { self.entries[idx].assume_init_drop() };
        self.free_stack[self.free_head as usize] = id;
        self.free_head += 1;
    }

    pub fn get(&self, id: u32) -> &Order {
        unsafe { self.entries[id as usize].assume_init_ref() }
    }

    pub fn get_mut(&mut self, id: u32) -> &mut Order {
        unsafe { self.entries[id as usize].assume_init_mut() }
    }

    pub fn len(&self) -> usize {
        self.next_id as usize
    }
}
