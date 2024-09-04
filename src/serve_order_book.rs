use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;

// Import relevant types
use crate::order_book::OrderBook;

// Serve the OrderBook as a simple webpage
pub async fn serve_order_book(
    order_book: Arc<RwLock<OrderBook>>,
) -> Result<impl warp::Reply, Infallible> {
    let order_book = order_book.read().await;

    // Format the OrderBook as HTML
    let buy_orders = order_book
        .buy_orders
        .iter()
        .map(|o| format!("<li>Buy: {} @ {}</li>", o.0.quantity, o.0.price))
        .collect::<Vec<String>>()
        .join("\n");

    let sell_orders = order_book
        .sell_orders
        .iter()
        .map(|o| format!("<li>Sell: {} @ {}</li>", o.quantity, o.price)) // o.0 is the inner order from Reverse
        .collect::<Vec<String>>()
        .join("\n");

    let html = format!(
        r#"
        <html>
        <head>
            <style>
                table {{
                    width: 50%;
                    border-collapse: collapse;
                    margin: 20px;
                }}
                table, th, td {{
                    border: 1px solid black;
                }}
                th, td {{
                    padding: 10px;
                    text-align: center;
                }}
            </style>
        </head>
        <body>
            <h1>Order Book</h1>
            <div style="display: flex; justify-content: space-around;">
                <div>
                    <h2>Bid</h2>
                    <table>
                        {}
                    </table>
                </div>
                <div>
                    <h2>Ask</h2>
                    <table>
                        {}
                    </table>
                </div>
            </div>
        </body>
        </html>
        "#,
        buy_orders, sell_orders
    );

    Ok(warp::reply::html(html))
}
