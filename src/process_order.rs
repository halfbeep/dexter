use serde_json::json;
use std::cmp::Reverse;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task;
use tokio::time::{sleep, Duration};

// Import relevant types
use crate::liquidity_pool::LiquidityPool;
use crate::order_book::{Order, OrderBook, OrderType};

pub async fn process_order(
    order: Order,
    order_book: Arc<RwLock<OrderBook>>,
    tx: mpsc::UnboundedSender<String>,
    liquidity_pool: Arc<RwLock<LiquidityPool>>,
) {
    // 1: Add the order to the order book immediately
    {
        let mut order_book_lock = order_book.write().await;
        order_book_lock.add_order(order.clone());
    }

    // 2: Spawn a separate task to handle matching and liquidity pool fallback
    let order_book_clone = Arc::clone(&order_book);
    let liquidity_pool_clone = Arc::clone(&liquidity_pool);
    let tx_clone = tx.clone();
    task::spawn(async move {
        let mut log_text = String::new();

        // Try matching orders in the order book, returning a tuple (bool, String)
        let (matched, match_log) = {
            let mut ob = order_book_clone.write().await;
            ob.match_orders()
        };

        log_text.push_str(&match_log);

        // 3: Handle liquidity pool if no match is found
        if !matched {
            println!("..delaying before using liquidity pool...");

            // Delay of 1 second for order-boook to form on start
            sleep(Duration::from_secs(1)).await;

            // Access and modify the liquidity pool (mutable access via write())
            let mut lp = liquidity_pool_clone.write().await;

            // Fallback to liquidity pool
            match order.order_type {
                OrderType::Buy => {
                    if let Some(received_b) = lp.swap_a_for_b(order.price.into_inner()) {
                        let mut ob = order_book_clone.write().await;
                        ob.buy_orders.take(&Reverse(order.clone())); // remove order from book !
                        let pool_log = format!(
                            "Order swapped in Liquidity Pool: Buy {:?} received {:?} token B",
                            order.quantity, received_b
                        );
                        log_text.push_str(&pool_log);
                        println!("{}", pool_log);
                    } else {
                        let pool_log =
                            "Swap couldn't be filled from the liquidity pool".to_string();
                        log_text.push_str(&pool_log);
                        println!("{}", pool_log);
                    }
                }
                OrderType::Sell => {
                    if let Some(received_a) = lp.swap_b_for_a(order.price.into_inner()) {
                        let mut ob = order_book_clone.write().await;
                        ob.sell_orders.take(&order); // remove order from book !
                        let pool_log = format!(
                            "Order swapped in Liquidity Pool: Sell {:?} received {:?} token A",
                            order.quantity, received_a
                        );
                        log_text.push_str(&pool_log);
                        println!("{}", pool_log);
                    } else {
                        let pool_log =
                            "Swap couldn't be filled from the liquidity pool".to_string();
                        log_text.push_str(&pool_log);
                        println!("{}", pool_log);
                    }
                }
            }
        }

        // Send the response as a JSON object to client
        // over the mpsc channel
        let response = json!({
            "status": log_text,
            "order": {
                "order_type": format!("{:?}", order.order_type),
                "price": order.price.into_inner(),
                "quantity": order.quantity
            }
        });

        if tx_clone.send(response.to_string()).is_err() {
            println!("Error sending response to client.");
        }
    });
}
