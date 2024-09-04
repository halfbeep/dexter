**Decentralized Exchange Test Env Release**

A basic exchange using WebSockets to connect clients with an order book and liquidity pool. Orders are matched with a simple off-chain engine for fast execution or swapped using AMM logic (x * y = k formula). A client can connect to the WebSocket server and send limit orders (buy/sell). dexter processes these orders, tries to match or swap them, and returns updates to the client

Use [dexter_client](https://github.com/halfbeep/dexter_client/blob/master/src/main.rs) to send orders 1000/second to this matching engine and swap pool for testing


![Decentralized Exchange Test Env Release ](https://pyxis.nymag.com/v1/imgs/715/9c7/86d1c07e8f6026d507e365bbafcd606ad9-23-dexter-1.rsquare.w400.jpg)|`{ "id": "ccdb079c-d33d-4539-bfe3-710f84d4230e", "order_type": "Buy", "price": 100.0, "quantity": 10 }`|
|--|--|

**Key components:**
- WebSocket Server: Allows communication between clients (traders) and the exchange
- Liquidity Pool: Swaps buy/sell orders with AMM tokens
- Order Matching Engine: Matches buy and sell orders based on price and quantity
- Basic Order Types: Support for limit orders (market orders may be added)

**To run:**
	bash
	
    git clone [this repo]
    cd dexter
    cargo run

The server will start at `ws://127.0.0.1:3030/ws`, also a basic html view of the order book can be seen at `http://127.0.0.1:8080`  (manual refresh)

Connect to the local ws server using web-sockets and send orders in the JSON format shown above next to photo. Use [dexter_client](https://github.com/halfbeep/dexter_client/blob/master/src/main.rs) for fast client config and testing
