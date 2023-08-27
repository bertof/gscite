use std::io::BufReader;
use std::sync::Arc;

use futures_util::TryStreamExt;
use gscite::{Client, QueryArgs, ReferenceFormat};
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cookie_store = {
        let reader = std::fs::File::open("./cookies.json").map(BufReader::new)?;
        let cs = reqwest_cookie_store::CookieStore::load_json(reader).unwrap_or_default();
        let cs = reqwest_cookie_store::CookieStoreMutex::new(cs);
        Arc::new(cs)
    };

    let client = Client::from(
        reqwest::ClientBuilder::new()
            .user_agent("Mozilla/5.0 (X11; Linux i686; rv:109.0) Gecko/20100101 Firefox/116.0")
            .cookie_store(true)
            .cookie_provider(cookie_store)
            .build()
            .unwrap(),
    );

    let results = client
        .get_references_with_query(
            QueryArgs {
                query: "Filippo Berto assurance",
                cite_id: Default::default(),
                from_year: Default::default(),
                to_year: Default::default(),
                sort_by: Default::default(),
                cluster_id: Default::default(),
                lang: Default::default(),
                lang_limit: Default::default(),
                limit: Default::default(),
                offset: Default::default(),
                adult_filtering: Default::default(),
                include_similar_results: Default::default(),
                include_citations: Default::default(),
            },
            ReferenceFormat::BibTeX,
        )
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    info!("Simple query results: {results:#?}");

    Ok(())
}
