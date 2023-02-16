use std::collections::HashSet;

use pickledb::PickleDb;
use reqwest::Response;
use scraper::{Html, Selector, ElementRef};
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use serenity::{futures::TryFutureExt, model::{webhook::Webhook, prelude::{Embed}}, http::{Http}, json::JsonMap};
use std::time::{Duration};

use crate::{errors::{SipError}, DATABASE, logger, };

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SipPostType {
    New = 0,
    Important = 1
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SipPost {
    pub post_type: SipPostType,
    pub date: String,
    pub title: String,
    pub content: String,
    pub link: String
}

impl PartialEq for SipPost {
    fn eq(&self, other: &Self) -> bool {
        self.link == other.link
    }
}

// setup selectors
lazy_static::lazy_static! {
    static ref ALL_POSTS_SELECTOR: Selector = Selector::parse("div.news-box").map_err(|err| SipError::SelectorError(err.to_string())).unwrap();
    static ref POSTS_SELECTOR: Selector = Selector::parse("ul li").map_err(|err| SipError::SelectorError(err.to_string())).unwrap();

    static ref DATE_SELECTOR: Selector = Selector::parse("p").map_err(|err| SipError::SelectorError(err.to_string())).unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse("h4").map_err(|err| SipError::SelectorError(err.to_string())).unwrap();
    static ref CONTENT_SELECTOR: Selector = Selector::parse("p").map_err(|err| SipError::SelectorError(err.to_string())).unwrap();
    static ref LINK_SELECTOR: Selector = Selector::parse("a").map_err(|err| SipError::SelectorError(err.to_string())).unwrap();
}

pub fn parse_element_to_posts(node: ElementRef, post_type: SipPostType) -> Result<Vec<SipPost>, SipError> {
    let mut posts: Vec<SipPost> = Vec::<SipPost>::new();

    for post_element in node.select(&POSTS_SELECTOR) {

        let date: String = post_element
                            .select(&DATE_SELECTOR)
                            .nth(0)
                            .ok_or_else(|| SipError::PostParseError("Error parsing date".to_string()))?
                            .text()
                            .map(|t| t.trim().to_string())
                            .collect();

        let title: String = post_element
                        .select(&TITLE_SELECTOR)
                        .nth(0)
                        .ok_or_else(|| SipError::PostParseError("Error parsing title".to_string()))?
                        .text()
                        .map(|t| t.trim().to_string())
                        .collect();

        let content: String = post_element
                        .select(&CONTENT_SELECTOR)
                        .nth(1)
                        .ok_or_else(|| SipError::PostParseError("Error parsing content".to_string()))?
                        .text()
                        .map(|t| t.trim().to_string())
                        .collect();

        let link: String = post_element
                        .select(&LINK_SELECTOR)
                        .nth(0)
                        .ok_or_else(|| SipError::PostParseError("Error parsing link".to_string()))?
                        .value()
                        .attr("href")
                        .ok_or_else(|| SipError::PostParseError("Error parsing hyperlink".to_string()))?
                        .trim()
                        .to_string();

        posts.push(SipPost { post_type: post_type.clone(), title, content, date, link });
    }

    return Ok(posts);
}

// vraca sve sip postove za slanje
pub async fn fetch_posts(database: &mut PickleDb) -> Result<Vec<SipPost>, SipError> {

    logger::log_sync("SPFCH", "READING POSTS");
    // linkovi starih postova, posto uklanjamo duplikate preko linkova
    let old_left_posts_links: HashSet<String> = database
                                                    .get::<Vec<SipPost>>("levi_stari")
                                                    .ok_or_else(|| SipError::StorageError("No left posts found".to_string()))?
                                                    .into_iter()
                                                    .map(|old_left_post| old_left_post.link)
                                                    .collect();

    let old_right_posts_links: HashSet<String> = database
                                                    .get::<Vec<SipPost>>("desni_stari")
                                                    .ok_or_else(|| SipError::StorageError("No right posts found".to_string()))?
                                                    .into_iter()
                                                    .map(|old_right_post| old_right_post.link)
                                                    .collect();
    logger::log_sync("SPFCH", "POSTS READ");



    // pribavljanje sa sip-a
    let response: Response = reqwest::get("https://sip.elfak.ni.ac.rs/").map_err(|err| SipError::FetchError(err.to_string())).await?;
    let html: String = response.text().map_err(|err| SipError::TextParseError(err.to_string())).await?;

    logger::log_sync("SPFCH", "FETCH ENDED");

    let document: Html = Html::parse_document(&html);

    logger::log_sync("SPFCH", "PARSE ENDED");

    let left_posts_element: ElementRef = document
                                            .select(&ALL_POSTS_SELECTOR)
                                            .nth(0)
                                            .ok_or_else(|| SipError::PostError("Missing posts".to_string()))?;
    let right_posts_element: ElementRef = document
                                            .select(&ALL_POSTS_SELECTOR)
                                            .nth(1)
                                            .ok_or_else(|| SipError::PostError("Missing posts".to_string()))?;

    // pribavljeni postovi sa sip-a
    let left_posts: Vec<SipPost> = parse_element_to_posts(left_posts_element, SipPostType::New).map_err(|err| SipError::PostParseError(err.to_string()))?;
    let right_posts: Vec<SipPost> = parse_element_to_posts(right_posts_element, SipPostType::Important).map_err(|err| SipError::PostParseError(err.to_string()))?;

    logger::log_sync("SPFCH", "SAVING POSTS");
    database.set("levi_stari", &left_posts).map_err(|err| SipError::StorageError(err.to_string()))?;
    database.set("desni_stari", &right_posts).map_err(|err| SipError::StorageError(err.to_string()))?;
    logger::log_sync("SPFCH", "POSTS SAVED");

    // izdvajanje novih postova
    let new_left_posts: Vec<SipPost> = left_posts
                                        .into_iter()
                                        .filter(|left_post| !old_left_posts_links.contains(&left_post.link))
                                        .collect();
    let new_right_posts: Vec<SipPost> = right_posts
                                            .into_iter()
                                            .filter(|right_post| !old_right_posts_links.contains(&right_post.link))
                                            .collect();

    // spajanje postova u jedan niz
    // uklanjanje dupliciranih novih postova
    // prioritet se daje desnim postovima
    // posto su oni vazna obavestenja
    let new_right_posts_links: HashSet<String> = new_right_posts.iter().map(|new_right_post| new_right_post.link.clone()).collect();
    let mut final_posts: Vec<SipPost> = new_right_posts;

    for new_left_post in new_left_posts {
        if !new_right_posts_links.contains(&new_left_post.link) {
            final_posts.push(new_left_post);
        }
    }

    logger::log_sync("SPFCH", format!("NEW POSTS: {}", final_posts.len()));

    return Ok(final_posts);
}

fn get_embed_author_name_from_post(post: &SipPost) -> String {
    return match post.post_type {
        SipPostType::New => "Најновије вести".to_string(),
        SipPostType::Important => "Важна обавештења".to_string()
    };
}

fn get_embed_color_from_post(post: &SipPost) -> i32 {
    return match post.post_type {
        SipPostType::New => 0x41662D,
        SipPostType::Important => 0x66F442
    };
}

pub fn create_embed_from_post(post: SipPost) -> Value {
    return Embed::fake(|e|
        e
            .author(|a|
                a
                    .name(get_embed_author_name_from_post(&post))
            )
            .color(get_embed_color_from_post(&post))
            .title(post.title)
            .description(post.content)
            .url(post.link)
            .thumbnail("https://i.imgur.com/dyu12dZ.png")
    );
}

pub async fn fetcher_main(http: Http) {
    let mut interval = tokio::time::interval(Duration::from_secs(15 * 60)); // 15 minuta

    loop {
        interval.tick().await;

        logger::log("SPFCH", "TASK STARTED").await;

        let webhooks: Vec<Webhook>;
        let embed_posts: Vec<Value>;

        {
            let mut database = DATABASE.lock().await;
            webhooks = database.get::<Vec<Webhook>>("sip_hooks").unwrap_or(Vec::<Webhook>::new());
            embed_posts = fetch_posts(&mut database).await.unwrap_or(Vec::<SipPost>::new()).into_iter().map(|sip_post| create_embed_from_post(sip_post)).collect();
        }

        let webhooks_count: usize = webhooks.len();
        let posts_count: usize = embed_posts.len();
        if webhooks_count == 0 || posts_count == 0  {
            logger::log("SPFCH", "TASK ENDED").await;
            continue;
        }

        // jedan webhook prima 10 embeda
        // moramo da podelimo posts na podnizove
        // od po 10 elementa
        // tako za svaki webhook

        let mut chunks: Vec<Value> = Vec::<Value>::new();
        for ten_chunk in embed_posts.chunks(10) {
            chunks.push(
                json!({"embeds": ten_chunk.to_owned()})
            );
        }
        let all_sessions_embeds: Vec<&JsonMap> = chunks.iter().filter_map(|x| x.as_object()).collect();

        for webhook in webhooks {
            if let Some(webhook_token) = webhook.token {
                for per_session_embed in all_sessions_embeds.iter() {
                    match http.execute_webhook(webhook.id.0, &webhook_token, true, &per_session_embed).await {
                        Ok(_) => {},
                        Err(why) => {
                            logger::log("ERR", why.to_string()).await;
                        }
                    };
                }
            }
        }

        logger::log("SPFCH", "TASK ENDED").await;
    }
}