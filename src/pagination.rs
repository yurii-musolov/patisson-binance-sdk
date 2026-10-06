//! Walk an id-ordered history endpoint (`allOrders` by `orderId`, trade
//! lists by `fromId`) page by page.
//!
//! Binance returns items with an id `>=` the requested one, oldest first.
//! The walk requests the next page from `last id + 1` and stops on an empty
//! page, so it doesn't depend on the endpoint's page size (one extra request
//! at the end). It also stops when the ids stop growing, and after
//! `max_pages` requests, returning where to continue.

use std::future::Future;

/// Items collected by an `*_all` method.
#[derive(Debug, Clone, PartialEq)]
pub struct AllPages<T> {
    /// Items in id order, each id once.
    pub items: Vec<T>,
    /// `Some(id)` when the walk stopped at `max_pages`: call again starting
    /// from this id to continue. `None` when the history is exhausted.
    pub next_id: Option<u64>,
}

/// Fetch pages from `start`; `fetch(from_id)` returns one page.
pub(crate) async fn walk_by_id<T, E, F, Fut>(
    start: u64,
    max_pages: usize,
    mut fetch: F,
    id: impl Fn(&T) -> u64,
) -> Result<AllPages<T>, E>
where
    F: FnMut(u64) -> Fut,
    Fut: Future<Output = Result<Vec<T>, E>>,
{
    let mut items: Vec<T> = Vec::new();
    let mut from = start;
    for _ in 0..max_pages {
        let page = fetch(from).await?;
        let Some(last) = page.iter().map(&id).max() else {
            return Ok(AllPages {
                items,
                next_id: None,
            });
        };
        // Keep only what wasn't seen: ids >= `from` (the server may repeat
        // or include older items).
        items.extend(page.into_iter().filter(|item| id(item) >= from));
        let next = last + 1;
        if next <= from {
            // No progress: don't loop forever on a misbehaving page.
            return Ok(AllPages {
                items,
                next_id: None,
            });
        }
        from = next;
    }
    Ok(AllPages {
        items,
        next_id: Some(from),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Server holding ids `1..=n`, pages of `size`.
    fn server(n: u64, size: usize) -> impl Fn(u64) -> Vec<u64> {
        move |from| (from.max(1)..=n).take(size).collect()
    }

    async fn walk(n: u64, size: usize, max_pages: usize) -> (AllPages<u64>, usize) {
        let calls = RefCell::new(0);
        let srv = server(n, size);
        let pages = walk_by_id::<_, (), _, _>(
            0,
            max_pages,
            |from| {
                *calls.borrow_mut() += 1;
                let page = srv(from);
                async move { Ok(page) }
            },
            |id| *id,
        )
        .await
        .unwrap();
        (pages, calls.into_inner())
    }

    #[tokio::test]
    async fn walks_every_page_once() {
        let (pages, calls) = walk(2500, 1000, 100).await;
        assert_eq!(pages.items, (1..=2500).collect::<Vec<_>>());
        assert_eq!(pages.next_id, None);
        assert_eq!(calls, 4, "3 pages + the empty one");
        // A server page smaller than expected doesn't end the walk early.
        let (pages, _) = walk(2500, 7, 1000).await;
        assert_eq!(pages.items.len(), 2500);
    }

    #[tokio::test]
    async fn stops_at_max_pages_and_says_where_to_continue() {
        let (pages, calls) = walk(2500, 1000, 2).await;
        assert_eq!(pages.items.len(), 2000);
        assert_eq!(pages.next_id, Some(2001));
        assert_eq!(calls, 2);
    }

    #[tokio::test]
    async fn repeated_items_and_no_progress() {
        // The server ignores `from` and always returns the same page.
        let pages =
            walk_by_id::<_, (), _, _>(5, 100, |_| async { Ok(vec![3u64, 4, 5, 6]) }, |id| *id)
                .await
                .unwrap();
        assert_eq!(pages.items, vec![5, 6], "older ids dropped");
        assert_eq!(pages.next_id, None, "stopped: no progress");
    }
}
