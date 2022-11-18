//! Google Scholar API client that queries for bibtex citation

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations
)]

use futures_util::{stream, Stream, StreamExt, TryStreamExt};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Url,
};
use scraper::{ElementRef, Html, Selector};

/// Query related errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error while parsing a URL
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    /// Error while sending a request
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
}

/// Type of citation to export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceFormat {
    /// BibTeX citation format
    BibTeX,
    /// EndNote citation format
    EndNote,
    /// RefMan citation format
    RefMan,
    /// RefWorks citation format
    RefWorks,
}

/// Google Scholar API Client
#[derive(Debug, Clone)]
pub struct Client(reqwest::Client);

impl Default for Client {
    fn default() -> Self {
        let headers = HeaderMap::from_iter([(
            HeaderName::from_static("referer"),
            HeaderValue::from_static("https://www.google.com/"),
        )]);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        Self(client)
    }
}

impl Client {
    /// Constructor with custom [`reqwest::Client`]
    #[must_use]
    pub fn with_client(client: reqwest::Client) -> Self {
        Client(client)
    }

    /// Get references for a given query
    ///
    /// # Errors
    /// Returns a [`Error::UrlParseError`] if the query generates an illegal [`Url`] or a [`Error::RequestError`] if the request fails.
    pub async fn get_references(
        &self,
        query: &str,
        format: ReferenceFormat,
    ) -> Result<impl Stream<Item = Result<String, Error>> + '_, Error> {
        let search_url = Self::get_search_url(query)?;
        let res = self.0.get(search_url).send().await?;
        let text = res.text().await?;
        let document = Html::parse_document(&text);
        let cit_ids = Self::scrape_citation_ids(&document)
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let references = stream::iter(cit_ids)
            .then(move |id: String| async move {
                let url = Self::get_cite_url(&id)?;
                let res = self.0.get(url).send().await?;
                let content = res.text().await?;
                let document = Html::parse_document(&content);
                let link = Self::scrape_citation_link(&document, format).to_string();
                let url = Url::parse(&link)?;
                Ok::<_, Error>(url)
            })
            .and_then(move |url: Url| async move {
                let reference = self.0.get(url).send().await?.text().await?;
                Ok(reference)
            });
        Ok(references)
    }

    /// Get search [`Url`] for a given query
    ///
    /// # Errors
    /// Returns a [`url::ParseError`] if the query produces an invalid URL
    pub(crate) fn get_search_url(query: &str) -> Result<Url, url::ParseError> {
        let mut url = Url::parse("https://scholar.google.com/scholar")?;

        url.query_pairs_mut()
            .append_pair("hl", "en")
            .append_pair("as_sdt", "0,5")
            .append_pair("q", query)
            .append_pair("btnG", "");

        Ok(url)
    }

    /// Scrapes an HTML document searching for citation ids
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub(crate) fn scrape_citation_ids(document: &Html) -> Vec<&str> {
        let block_sel = Selector::parse("div.gs_ri").unwrap();
        let title_sel = Selector::parse("h3").unwrap();
        let link_sel = Selector::parse("a").unwrap();

        document
            .select(&block_sel)
            .flat_map(|block: ElementRef| block.select(&title_sel))
            .flat_map(|title: ElementRef| title.select(&link_sel))
            .filter_map(|link: ElementRef| link.value().attr("id"))
            .collect::<Vec<_>>()
    }

    /// Get citation URL for a given citation id
    ///
    /// # Errors
    /// Returns a [`url::ParseError`] if the query produces an invalid URL.
    /// This should never happen with the ids obtained by [`Self::scrape_citation_ids`]
    pub(crate) fn get_cite_url(citation_id: &str) -> Result<Url, url::ParseError> {
        let mut url = Url::parse("https://scholar.google.com/scholar")?;
        let query = format!("info:{}:scholar.google.com/", citation_id);

        url.query_pairs_mut()
            .append_pair("hl", "en")
            .append_pair("q", query.as_str())
            .append_pair("output", "cite")
            .append_pair("scirp", "0");

        Ok(url)
    }

    /// Scrapes an HTML document searching for a BibTeX entry URL
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub(crate) fn scrape_citation_link(document: &Html, format: ReferenceFormat) -> &str {
        let citation_sel = Selector::parse("div#gs_citi").unwrap();
        let link_sel = Selector::parse("a").unwrap();
        let format = match format {
            ReferenceFormat::BibTeX => "BibTeX",
            ReferenceFormat::EndNote => "EndNote",
            ReferenceFormat::RefMan => "RefMan",
            ReferenceFormat::RefWorks => "RefWorks",
        };

        document
            .select(&citation_sel)
            .flat_map(|citation: ElementRef| citation.select(&link_sel))
            .find(|a: &ElementRef| a.inner_html() == format)
            .and_then(|link: ElementRef| link.value().attr("href"))
            .unwrap()
    }
}

/// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_search_url() {
        let url = Client::get_search_url("security assurance").unwrap();
        println!("QUERY URL: {}", url);
        assert_eq!(
            url,
            Url::parse(
                "https://scholar.google.com/scholar?hl=en&as_sdt=0%2C5&q=security+assurance&btnG= "
            )
            .unwrap()
        );
    }

    #[test]
    fn scrape_citation_ids() {
        let content = include_str!("../samples/query_response.html");
        let document = Html::parse_document(content);
        let ids = Client::scrape_citation_ids(&document);
        println!("IDS: {:?}", ids);
        assert_eq!(
            ids,
            vec![
                "oRnsanDfyFAJ",
                "h04c3ps-QG4J",
                "K1ufdskeGhoJ",
                "oSQ2ikcD5YUJ",
                "kWdqyvppSk4J",
                "ga0OyWXd7jYJ",
                "PsyfzHL8y6sJ",
                "vx9FMpr8xsoJ",
                "PH5yhK_1--EJ",
                "3nA3AEXeAgsJ"
            ]
        );
    }

    #[test]
    fn get_cite_url() {
        let url = Client::get_cite_url("oRnsanDfyFAJ").unwrap();
        println!("CITE URL: {}", url);
        assert_eq!(
            url,
            Url::parse(
                "https://scholar.google.com/scholar?hl=en&q=info%3AoRnsanDfyFAJ%3Ascholar.google.com%2F&output=cite&scirp=0"
            )
            .unwrap()
        );
    }

    #[test]
    fn scrape_citation_link() {
        let content = include_str!("../samples/cite_response.html");
        let document = Html::parse_document(content);
        assert_eq!(Client::scrape_citation_link(&document, ReferenceFormat::BibTeX),  "https://scholar.googleusercontent.com/scholar.bib?q=info:oRnsanDfyFAJ:scholar.google.com/&output=citation&scisdr=CgXc7mXxEJuhju7JwnE:AAGBfm0AAAAAY3bP2nFwv5yvzTHsok6iOzPciqpmgQNn&scisig=AAGBfm0AAAAAY3bP2gGBvu6qzVeapAa4iOTHNZWb5QQy&scisf=4&ct=citation&cd=-1&hl=en");
        assert_eq!(Client::scrape_citation_link(&document, ReferenceFormat::EndNote), "https://scholar.googleusercontent.com/scholar.enw?q=info:oRnsanDfyFAJ:scholar.google.com/&output=citation&scisdr=CgXc7mXxEJuhju7JwnE:AAGBfm0AAAAAY3bP2nFwv5yvzTHsok6iOzPciqpmgQNn&scisig=AAGBfm0AAAAAY3bP2gGBvu6qzVeapAa4iOTHNZWb5QQy&scisf=3&ct=citation&cd=-1&hl=en");
        assert_eq!(Client::scrape_citation_link(&document, ReferenceFormat::RefMan),  "https://scholar.googleusercontent.com/scholar.ris?q=info:oRnsanDfyFAJ:scholar.google.com/&output=citation&scisdr=CgXc7mXxEJuhju7JwnE:AAGBfm0AAAAAY3bP2nFwv5yvzTHsok6iOzPciqpmgQNn&scisig=AAGBfm0AAAAAY3bP2gGBvu6qzVeapAa4iOTHNZWb5QQy&scisf=2&ct=citation&cd=-1&hl=en");
        assert_eq!(Client::scrape_citation_link(&document, ReferenceFormat::RefWorks),"https://scholar.googleusercontent.com/scholar.rfw?q=info:oRnsanDfyFAJ:scholar.google.com/&output=citation&scisdr=CgXc7mXxEJuhju7JwnE:AAGBfm0AAAAAY3bP2nFwv5yvzTHsok6iOzPciqpmgQNn&scisig=AAGBfm0AAAAAY3bP2gGBvu6qzVeapAa4iOTHNZWb5QQy&scisf=1&ct=citation&cd=-1&hl=en");
    }

    #[tokio::test]
    async fn query_results() {
        let client = Client::default();
        let results = client
            .get_references("Filippo Berto Assurance", ReferenceFormat::BibTeX)
            .await
            .unwrap();

        let references = results.take(1).try_collect::<Vec<_>>().await.unwrap();

        for r in references {
            println!("{}", r);
        }
    }
}
