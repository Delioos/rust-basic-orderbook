# Basic Order Book

A simple order matching engine implemented in Rust that simulates a financial trading system.

## Overview

This project implements a basic order matching engine that processes buy and sell orders for financial instruments. It demonstrates core concepts of financial trading systems including:

- Order book management
- Price-time priority matching
- Bid-ask spread calculation
- Trade execution

## Features

- **Order Book**: Maintains separate collections for buy and sell orders, sorted by price
- **Price-Time Priority**: Orders are matched based on best price first, then by time of arrival
- **Partial Matching**: Orders can be partially filled with the remainder staying in the book
- **Random Order Generation**: Simulates trading activity with randomly generated orders
- **Real-time Display**: Shows the current state of the order book after each transaction

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/basic-order-book.git
   cd basic-order-book
   ```

2. Build the project:
   ```
   cargo build
   ```

3. Run the simulation:
   ```
   cargo run
   ```

## How It Works

The system consists of several key components:

1. **Order**: Represents a buy or sell request with a specific price and quantity
2. **OrderBook**: The central component that matches orders and maintains the state
3. **Trade**: Represents a completed transaction between a buy and sell order

When a new order is placed:
- For buy orders: The system tries to match with existing sell orders at the lowest available price
- For sell orders: The system tries to match with existing buy orders at the highest available price
- Any remaining quantity is added to the order book

## Example Output

```
Simulating random trading for AAPL
=================================

Round 1
Placing Sell order: 20 shares of AAPL at $106
Order Book for AAPL
---------------------------
SELL ORDERS:
  106: 20 shares
---------------------------
BUY ORDERS:
---------------------------
Best ask: $106 (no bids)

Round 2
Placing Buy order: 8 shares of AAPL at $102
Order Book for AAPL
---------------------------
SELL ORDERS:
  106: 20 shares
---------------------------
BUY ORDERS:
  102: 8 shares
---------------------------
Current spread: $106 - $102 = $4
```

## Future Enhancements

- Support for multiple trading symbols
- Order cancellation functionality
- Different order types (market, limit, stop, etc.)
- Performance optimizations for high-frequency trading
- REST API for external order submission

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by real-world financial trading systems
- Built as an educational tool to understand order matching algorithms 