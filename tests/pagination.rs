//! `*_all` methods walk a local HTTP server that pages like Binance:
//! items with id >= orderId / fromId, oldest first, at most `limit`.

use binance::{
    SensitiveString,
    spot::http::{GetAccountTradeListParams, GetAllOrdersParams, PrivateClient, PrivateConfig},
};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/spot/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn query_param(target: &str, name: &str) -> Option<u64> {
    target
        .split(['?', '&'])
        .find_map(|kv| kv.strip_prefix(&format!("{name}=")))
        .and_then(|v| v.parse().ok())
}

/// Serve `total` items made from `template` (ids 1..=total, `id_field`)
/// and report each request's `from` parameter.
async fn history_server(
    template: Value,
    id_field: &'static str,
    from_param: &'static str,
    total: u64,
    server_page_cap: u64,
) -> (String, mpsc::UnboundedReceiver<u64>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Ok((mut tcp, _)) = listener.accept().await {
            let template = template.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut head = Vec::new();
                while !head.ends_with(b"\r\n\r\n") {
                    match tcp.read_u8().await {
                        Ok(b) => head.push(b),
                        Err(_) => return,
                    }
                }
                let head = String::from_utf8(head).unwrap();
                let target = head.split_whitespace().nth(1).unwrap().to_string();
                let from = query_param(&target, from_param).unwrap_or(0);
                let limit = query_param(&target, "limit").unwrap_or(500);
                let _ = tx.send(from);
                let page: Vec<Value> = (from.max(1)..=total)
                    .take(limit.min(server_page_cap) as usize)
                    .map(|id| {
                        let mut item = template.clone();
                        item[id_field] = json!(id);
                        item
                    })
                    .collect();
                let body = serde_json::to_string(&page).unwrap();
                let reply = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = tcp.write_all(reply.as_bytes()).await;
            });
        }
    });
    (url, rx)
}

fn client(url: String) -> PrivateClient {
    let cfg = PrivateConfig::new(url, SensitiveString::from("k"), SensitiveString::from("s"));
    PrivateClient::new(cfg).unwrap()
}

#[tokio::test]
async fn spot_all_orders_walks_by_order_id() {
    let template = fixture("all_orders.json")[0].clone();
    let (url, mut froms) = history_server(template, "orderId", "orderId", 2345, 1000).await;
    let pages = client(url)
        .get_all_orders_all(GetAllOrdersParams::new("LTCBTC"), 100)
        .await
        .unwrap();
    let ids: Vec<i64> = pages.items.iter().map(|o| o.order_id).collect();
    assert_eq!(ids, (1..=2345).collect::<Vec<_>>());
    assert_eq!(pages.next_id, None);
    let mut requested = Vec::new();
    while let Ok(from) = froms.try_recv() {
        requested.push(from);
    }
    assert_eq!(requested, vec![0, 1001, 2001, 2346]);
}

#[tokio::test]
async fn spot_trades_resume_after_max_pages() {
    let template = fixture("my_trades.json")[0].clone();
    // The server caps pages at 300 although 1000 are requested.
    let (url, _) = history_server(template, "id", "fromId", 1000, 300).await;
    let client = client(url);
    let first = client
        .get_account_trade_list_all(GetAccountTradeListParams::new("BNBBTC"), 2)
        .await
        .unwrap();
    assert_eq!(first.items.len(), 600);
    assert_eq!(first.next_id, Some(601));
    let rest = client
        .get_account_trade_list_all(
            GetAccountTradeListParams::new("BNBBTC").from_id(first.next_id.unwrap() as i64),
            100,
        )
        .await
        .unwrap();
    assert_eq!(rest.items.len(), 400);
    assert_eq!(rest.items.first().unwrap().id, 601);
    assert_eq!(rest.next_id, None);
}
