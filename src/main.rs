use dexter::liquidity_pool::LiquidityPool;
use dexter::order_book::{Order, OrderBook};
use dexter::process_order::process_order;
use dexter::serve_order_book::serve_order_book;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{self, Duration};
use warp::Filter;

#[tokio::main]
async fn main() {
    // liquidity pool start-up (both sides)
    let token_count: f64 = 1000000000.0;

    // Initialize the Shared OrderBook and Liquidity pool wrapped in Arc and RwLock for safe concurrent access
    let order_book = Arc::new(RwLock::new(OrderBook::new()));
    let liquidity_pool = Arc::new(RwLock::new(LiquidityPool::new(100000.0, 100000.0)));
    println!("Liquidity pool running with {} tokens", token_count);

    // Clone the order book to use in the matching engine loop
    let order_book_clone = Arc::clone(&order_book);

    // Clone for WebSocket and HTTP routes
    let ws_order_book = Arc::clone(&order_book);
    let ws_liquidity_pool = Arc::clone(&liquidity_pool);

    // WebSocket route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || Arc::clone(&ws_order_book))) // Clone the order_book
        .and(warp::any().map(move || Arc::clone(&ws_liquidity_pool))) // Clone the liquidity_pool
        .map(
            |ws: warp::ws::Ws,
             order_book: Arc<RwLock<OrderBook>>,
             liquidity_pool: Arc<RwLock<LiquidityPool>>| {
                ws.on_upgrade(move |socket| handle_connection(socket, order_book, liquidity_pool))
            },
        );

    // HTTP route to serve the order book as a webpage
    let http_order_book = Arc::clone(&order_book);
    let http_route = warp::path::end()
        .and(warp::any().map(move || Arc::clone(&http_order_book)))
        .and_then(serve_order_book);

    // WebSocket server running on port 3030
    tokio::spawn(async move {
        println!("WebSocket server running on ws://127.0.0.1:3030/ws");
        warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
    });

    // Start the HTTP server in a separate async task
    tokio::spawn(async move {
        println!("HTTP server running on http://127.0.0.1:8080");
        warp::serve(http_route).run(([127, 0, 0, 1], 8080)).await;
    });

    // Run the matching engine 100 times per second
    let mut interval = time::interval(Duration::from_millis(10)); // 10 ms interval = 100 times/sec
    loop {
        interval.tick().await; // Wait for the next tick
        let mut order_book = order_book_clone.write().await; // Await the write lock
        order_book.match_orders(); // Run the matching engine
    }
}

// Handle WebSocket connection
async fn handle_connection(
    ws: warp::ws::WebSocket,
    order_book: Arc<RwLock<OrderBook>>,
    lp: Arc<RwLock<LiquidityPool>>,
) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    println!("Client connected..");

    // Spawn a task to handle sending messages to the client
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if ws_tx.send(warp::ws::Message::text(message)).await.is_err() {
                println!("Error sending message to client.");
                break;
            }
        }
    });

    // Handle receiving messages from the client
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    println!("Received message: {}", text); // Log message to console
                    if let Ok(order) = serde_json::from_str::<Order>(text) {
                        process_order(order, order_book.clone(), tx.clone(), lp.clone()).await;
                    } else {
                        println!("Error parsing message to Order struct.");
                    }
                } else {
                    println!("Received non-text message.");
                }
            }
            Err(e) => {
                println!("Error receiving WebSocket message: {}", e);
                break;
            }
        }
    }

    println!("Client disconnected..");
}
