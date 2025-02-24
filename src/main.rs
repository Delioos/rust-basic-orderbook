use std::collections::{BTreeMap, HashMap};
use std::fmt;
use chrono::{DateTime, Utc};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use uuid::Uuid;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    id: String,
    trader_id: String,
    symbol: String,
    price: u64,
    quantity: u64,
    side: Side,
    timestamp: DateTime<Utc>,
}

impl Order {
    pub fn new(trader_id: String, symbol: String, price: u64, quantity: u64, side: Side) -> Self {
        Order {
            id: Uuid::new_v4().to_string(),
            trader_id,
            symbol,
            price,
            quantity,
            side,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Trade {
    id: String,
    buy_order_id: String,
    sell_order_id: String,
    symbol: String,
    price: u64,
    quantity: u64,
    timestamp: DateTime<Utc>,
}

pub struct OrderBook {
    symbol: String,
    buy_orders: BTreeMap<u64, Vec<Order>>,
    sell_orders: BTreeMap<u64, Vec<Order>>,
    orders_by_id: HashMap<String, Order>,
    trades: Vec<Trade>,
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        OrderBook {
            symbol,
            buy_orders: BTreeMap::new(),
            sell_orders: BTreeMap::new(),
            orders_by_id: HashMap::new(),
            trades: Vec::new(),
        }
    }

    pub fn place_order(&mut self, order: Order) -> Vec<Trade> {
        if order.symbol != self.symbol {
            panic!("Order symbol does not match orderbook symbol");
        }

        let mut trades = Vec::new();
        let mut remaining_order = order.clone();

        match order.side {
            Side::Buy => {
                // Try to match with existing sell orders
                while remaining_order.quantity > 0 {
                    // Get the best (lowest) sell price
                    let best_sell_price_opt = self.sell_orders.keys().next().cloned();
                    
                    match best_sell_price_opt {
                        Some(best_sell_price) if best_sell_price <= remaining_order.price => {
                            let sell_orders = self.sell_orders.get_mut(&best_sell_price).unwrap();
                            
                            // Try to match with sell orders at this price level
                            while !sell_orders.is_empty() && remaining_order.quantity > 0 {
                                let mut sell_order = sell_orders[0].clone();
                                
                                // Calculate trade quantity
                                let trade_quantity = std::cmp::min(remaining_order.quantity, sell_order.quantity);
                                
                                // Create trade
                                let trade = Trade {
                                    id: Uuid::new_v4().to_string(),
                                    buy_order_id: remaining_order.id.clone(),
                                    sell_order_id: sell_order.id.clone(),
                                    symbol: self.symbol.clone(),
                                    price: best_sell_price,
                                    quantity: trade_quantity,
                                    timestamp: Utc::now(),
                                };
                                
                                trades.push(trade);
                                
                                // Update remaining quantities
                                remaining_order.quantity -= trade_quantity;
                                sell_order.quantity -= trade_quantity;
                                
                                // Update or remove the matched sell order
                                if sell_order.quantity == 0 {
                                    sell_orders.remove(0);
                                    self.orders_by_id.remove(&sell_order.id);
                                } else {
                                    sell_orders[0] = sell_order.clone();
                                    self.orders_by_id.insert(sell_order.id.clone(), sell_order);
                                }
                            }
                            
                            // If no sell orders left at this price, remove the price level
                            if sell_orders.is_empty() {
                                self.sell_orders.remove(&best_sell_price);
                            }
                        },
                        _ => break, // No matching sell orders, or price is too high
                    }
                }
                
                // If there's still quantity remaining, add to the buy orders
                if remaining_order.quantity > 0 {
                    self.add_buy_order(remaining_order);
                }
            },
            Side::Sell => {
                // Try to match with existing buy orders
                while remaining_order.quantity > 0 {
                    // Get the best (highest) buy price
                    let best_buy_price_opt = self.buy_orders.keys().next_back().cloned();
                    
                    match best_buy_price_opt {
                        Some(best_buy_price) if best_buy_price >= remaining_order.price => {
                            let buy_orders = self.buy_orders.get_mut(&best_buy_price).unwrap();
                            
                            // Try to match with buy orders at this price level
                            while !buy_orders.is_empty() && remaining_order.quantity > 0 {
                                let mut buy_order = buy_orders[0].clone();
                                
                                // Calculate trade quantity
                                let trade_quantity = std::cmp::min(remaining_order.quantity, buy_order.quantity);
                                
                                // Create trade
                                let trade = Trade {
                                    id: Uuid::new_v4().to_string(),
                                    buy_order_id: buy_order.id.clone(),
                                    sell_order_id: remaining_order.id.clone(),
                                    symbol: self.symbol.clone(),
                                    price: best_buy_price,
                                    quantity: trade_quantity,
                                    timestamp: Utc::now(),
                                };
                                
                                trades.push(trade);
                                
                                // Update remaining quantities
                                remaining_order.quantity -= trade_quantity;
                                buy_order.quantity -= trade_quantity;
                                
                                // Update or remove the matched buy order
                                if buy_order.quantity == 0 {
                                    buy_orders.remove(0);
                                    self.orders_by_id.remove(&buy_order.id);
                                } else {
                                    buy_orders[0] = buy_order.clone();
                                    self.orders_by_id.insert(buy_order.id.clone(), buy_order);
                                }
                            }
                            
                            // If no buy orders left at this price, remove the price level
                            if buy_orders.is_empty() {
                                self.buy_orders.remove(&best_buy_price);
                            }
                        },
                        _ => break, // No matching buy orders, or price is too low
                    }
                }
                
                // If there's still quantity remaining, add to the sell orders
                if remaining_order.quantity > 0 {
                    self.add_sell_order(remaining_order);
                }
            },
        }

        // Add trades to the orderbook
        self.trades.extend(trades.clone());
        
        trades
    }

    fn add_buy_order(&mut self, order: Order) {
        let price = order.price;
        self.orders_by_id.insert(order.id.clone(), order.clone());
        
        self.buy_orders
            .entry(price)
            .or_insert_with(Vec::new)
            .push(order);
    }

    fn add_sell_order(&mut self, order: Order) {
        let price = order.price;
        self.orders_by_id.insert(order.id.clone(), order.clone());
        
        self.sell_orders
            .entry(price)
            .or_insert_with(Vec::new)
            .push(order);
    }

    pub fn get_best_bid(&self) -> Option<u64> {
        self.buy_orders.keys().next_back().cloned()
    }

    pub fn get_best_ask(&self) -> Option<u64> {
        self.sell_orders.keys().next().cloned()
    }

    pub fn display_order_book(&self) {
        println!("Order Book for {}", self.symbol);
        println!("---------------------------");
        
        println!("SELL ORDERS:");
        let sell_prices: Vec<_> = self.sell_orders.keys().collect();
        for &price in sell_prices.iter().rev() {
            let orders = &self.sell_orders[price];
            let total_quantity: u64 = orders.iter().map(|order| order.quantity).sum();
            println!("  {}: {} shares", price, total_quantity);
        }
        
        println!("---------------------------");
        
        println!("BUY ORDERS:");
        let buy_prices: Vec<_> = self.buy_orders.keys().collect();
        for &price in buy_prices.iter().rev() {
            let orders = &self.buy_orders[price];
            let total_quantity: u64 = orders.iter().map(|order| order.quantity).sum();
            println!("  {}: {} shares", price, total_quantity);
        }
        
        println!("---------------------------");
    }
}

fn generate_random_trader_id() -> String {
    let mut rng = thread_rng();
    format!("TRADER-{}", 
        std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .take(5)
        .map(char::from)
        .collect::<String>()
    )
}

fn generate_random_order(symbol: &str) -> Order {
    let mut rng = thread_rng();
    
    let side = if rng.gen_bool(0.5) { Side::Buy } else { Side::Sell };
    
    // Generate price between 90 and 110
    let base_price = 100;
    let price_variation = rng.gen_range(-10..=10);
    let price = (base_price + price_variation) as u64;
    
    // Generate quantity between 1 and 20
    let quantity = rng.gen_range(1..=20);
    
    Order::new(
        generate_random_trader_id(),
        symbol.to_string(),
        price,
        quantity,
        side,
    )
}

fn main() {
    let symbol = "AAPL";
    let mut order_book = OrderBook::new(symbol.to_string());
    
    println!("Simulating random trading for {}", symbol);
    println!("=================================");
    
    for i in 1..=10 {
        println!("\nRound {}", i);
        
        // Generate a random order
        let order = generate_random_order(symbol);
        println!("Placing {:?} order: {} shares of {} at ${}", 
            order.side, order.quantity, symbol, order.price);
        
        // Place the order and get any resulting trades
        let trades = order_book.place_order(order);
        
        // Report any trades that occurred
        if !trades.is_empty() {
            println!("TRADES EXECUTED:");
            for trade in &trades {
                println!("  {} shares at ${}", trade.quantity, trade.price);
            }
        }
        
        // Display the current order book
        order_book.display_order_book();
        
        // Show the current spread
        let best_bid = order_book.get_best_bid().unwrap_or(0);
        let best_ask = order_book.get_best_ask().unwrap_or(0);
        
        if best_bid > 0 && best_ask > 0 {
            println!("Current spread: ${} - ${} = ${}", 
                best_ask, best_bid, best_ask.saturating_sub(best_bid));
        } else if best_bid > 0 {
            println!("Best bid: ${} (no asks)", best_bid);
        } else if best_ask > 0 {
            println!("Best ask: ${} (no bids)", best_ask);
        } else {
            println!("Order book is empty");
        }
        
        // Add a delay between rounds (2 seconds)
        if i < 10 {
            println!("\nWaiting for next round...");
            thread::sleep(Duration::from_secs(2));
        }
    }
}
