use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::error;
use tracing_subscriber::EnvFilter;

use crawl_ipc::CrawlEvent;

use crawl_mail::{accounts, config, ipc, smtp, store};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = config::load()?;

    let db = store::open(&cfg.db_path).await?;
    store::migrate(&db).await?;
    let store = Arc::new(store::Store::new(db));

    let (push_tx, _) = broadcast::channel::<CrawlEvent>(256);

    let account_mgr = Arc::new(accounts::AccountManager::new(
        Arc::clone(&store),
        push_tx.clone(),
    ));
    account_mgr.start_all().await?;

    let queue = smtp::SmtpQueue::new((*store).clone());
    tokio::spawn(async move {
        if let Err(e) = queue.run().await {
            error!("SMTP queue exited: {}", e);
        }
    });

    let socket_path = cfg.socket_path.clone();
    ipc::serve(
        socket_path,
        Arc::clone(&store),
        Arc::clone(&account_mgr),
        push_tx,
    )
    .await?;

    Ok(())
}
