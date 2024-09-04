use ordered_float::OrderedFloat;
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub struct Order {
    pub id: Uuid,
    pub order_type: OrderType,
    #[serde(
        serialize_with = "ordered_float_serialize",
        deserialize_with = "ordered_float_deserialize"
    )]
    pub price: OrderedFloat<f64>, // Price field needs custom serialization
    pub quantity: u64,
}

// Custom serialization for OrderedFloat<f64>
fn ordered_float_serialize<S>(price: &OrderedFloat<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(price.into_inner())
}

// Custom deserialization for OrderedFloat<f64>
fn ordered_float_deserialize<'de, D>(deserializer: D) -> Result<OrderedFloat<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let price = f64::deserialize(deserializer)?;
    Ok(OrderedFloat(price))
}

// Implement Ord and PartialOrd to order by price, and use id for uniqueness
impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price
            .cmp(&other.price) // Compare by price first
            .then_with(|| self.id.cmp(&other.id)) // Break ties using id
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.id == other.id
    }
}

impl Order {
    pub fn new(order_type: OrderType, price: f64, quantity: u64) -> Self {
        Order {
            id: Uuid::new_v4(),
            order_type,
            price: OrderedFloat(price),
            quantity,
        }
    }
}

#[derive(Debug)]
pub struct OrderBook {
    pub buy_orders: BTreeSet<Reverse<Order>>, // Fully ordered Buy Orders (Reversed so highest buy price at top of boook)
    pub sell_orders: BTreeSet<Order>, // Fully ordered Sell Orders (min-heap behavior asc down)
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            buy_orders: BTreeSet::new(),
            sell_orders: BTreeSet::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        match order.order_type {
            OrderType::Buy => {
                self.buy_orders.insert(Reverse(order)); // BTreeSet automatically orders by price (Reversed)
            }
            OrderType::Sell => {
                self.sell_orders.insert(order); //  min-heap behavior
            }
        }
    }

    pub fn match_orders(&mut self) -> (bool, String) {
        while let Some(Reverse(buy_order)) = self.buy_orders.iter().next().cloned() {
            if let Some(sell_order) = self.sell_orders.iter().next().cloned() {
                if buy_order.price >= sell_order.price {
                    println!(
                        "Matched Buy Order: {:?} with Sell Order: {:?}",
                        buy_order, sell_order
                    );

                    // Clone the buy_order before removing it from the BTreeSet
                    self.buy_orders.take(&Reverse(buy_order.clone()));
                    self.sell_orders.take(&sell_order.clone());

                    // Return success and log message
                    return (true, format!(
                        "Matched Buy Order (price: {}, quantity: {}) with Sell Order (price: {}, quantity: {})",
                        buy_order.price, buy_order.quantity, sell_order.price, sell_order.quantity
                    ));
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        // No match found, return false with a log message
        (false, "No match found in the order book".to_string())
    }
}
