//! Calls to newsletter-system on member changes.
//!
//! Every call forwards the acting admin's token and never fails the orca
//! action: errors are only logged, newsletter-system's daily reconciliation
//! (`GET /sync/newsletter/members`) catches anything missed.

use std::time::Duration;

use log::{debug, error};
use reqwest::{Client, Method};
use serde::Serialize;

use crate::data::{Id, Member, MemberNumber, Workplace};
use crate::db::DbPool;
use crate::server::oid::JwtToken;

mod query;

/// Member payload for newsletter-system. Field set is a contract: send nothing else.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NewsletterMember {
    pub(crate) member_number: MemberNumber,
    pub(crate) email: String,
    /// First name and last name joined with one space
    pub(crate) name: String,
    pub(crate) city: Option<String>,
    pub(crate) lang: String,
    /// `workplaces.newsletter_id` (a Listmonk list ID) of the member's workplace
    pub(crate) workplace_newsletter_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Newsletter {
    client: Client,
    /// `{newsletter_host}/api`; `None` disables all calls (development)
    api_url: Option<String>,
}

impl Newsletter {
    /// # Panics
    ///
    /// When the HTTP client can't be built (TLS backend failure at startup)
    #[must_use]
    pub fn new(newsletter_host: Option<&str>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("newsletter HTTP client");

        let api_url = newsletter_host
            .map(|host| host.trim_end_matches('/'))
            .filter(|host| !host.is_empty())
            .map(|host| format!("{host}/api"));

        Self { client, api_url }
    }

    /// `POST /api/members` for a newly accepted member
    pub async fn create_member(&self, pool: &DbPool, token: &JwtToken, id: Id<Member>) {
        let Some(api_url) = &self.api_url else { return };
        let Some(member) = fetch_member(pool, id).await else {
            return;
        };
        let url = format!("{api_url}/members");
        self.send(
            Method::POST,
            &url,
            token,
            member.member_number,
            Some(&member),
        )
        .await;
    }

    /// `PUT /api/members/{member_number}` after a member's data changed
    pub async fn update_member(&self, pool: &DbPool, token: &JwtToken, id: Id<Member>) {
        if self.api_url.is_none() {
            return;
        }
        if let Some(member) = fetch_member(pool, id).await {
            self.put_member(token, &member).await;
        }
    }

    /// `PUT` for every member of a workplace after its `newsletter_id` changed
    /// This can send a lot of requests, 1 per workplace member. We don't expect
    /// this situation to happen often, but if it does, we can prepare bulk EP
    /// instead of calling update in the loop.
    pub async fn update_workplace_members(
        &self,
        pool: &DbPool,
        token: &JwtToken,
        workplace_id: Id<Workplace>,
    ) {
        if self.api_url.is_none() {
            return;
        }
        match query::list_members(pool, None, Some(workplace_id)).await {
            Ok(members) => {
                for member in &members {
                    self.put_member(token, member).await;
                }
            }
            Err(err) => error!("Newsletter: can't load members of workplace {workplace_id}: {err}"),
        }
    }

    /// `DELETE /api/members/{member_number}` when a member leaves
    pub async fn delete_member(&self, token: &JwtToken, member_number: MemberNumber) {
        let Some(api_url) = &self.api_url else { return };
        let url = format!("{api_url}/members/{member_number}");
        self.send::<()>(Method::DELETE, &url, token, member_number, None)
            .await;
    }

    async fn put_member(&self, token: &JwtToken, member: &NewsletterMember) {
        let Some(api_url) = &self.api_url else { return };
        let body = match put_body(member) {
            Ok(body) => body,
            Err(err) => {
                error!(
                    "Newsletter: can't serialize member {}: {err}",
                    member.member_number
                );
                return;
            }
        };
        let url = format!("{api_url}/members/{}", member.member_number);
        self.send(Method::PUT, &url, token, member.member_number, Some(&body))
            .await;
    }

    async fn send<T: Serialize + ?Sized>(
        &self,
        method: Method,
        url: &str,
        token: &JwtToken,
        member_number: MemberNumber,
        body: Option<&T>,
    ) {
        let mut request = self
            .client
            .request(method.clone(), url)
            .bearer_auth(token.as_str());
        if let Some(body) = body {
            request = request.json(body);
        }

        match request.send().await {
            Ok(res) if res.status().is_success() => {
                debug!(
                    "Newsletter: {method} {url} for member {member_number}: {}",
                    res.status()
                );
            }
            Ok(res) => {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                error!(
                    "Newsletter: {method} {url} for member {member_number} failed with {status}: {text}"
                );
            }
            Err(err) => {
                error!("Newsletter: {method} {url} for member {member_number} failed: {err}");
            }
        }
    }
}

/// `PUT` takes the payload without `member_number` (it's in the path)
fn put_body(member: &NewsletterMember) -> serde_json::Result<serde_json::Value> {
    let mut body = serde_json::to_value(member)?;
    if let Some(fields) = body.as_object_mut() {
        fields.remove("member_number");
    }
    Ok(body)
}

/// `None` for a member who left or doesn't exist; there's nothing to send then.
async fn fetch_member(pool: &DbPool, id: Id<Member>) -> Option<NewsletterMember> {
    match query::list_members(pool, Some(id), None).await {
        Ok(members) => members.into_iter().next(),
        Err(err) => {
            error!("Newsletter: can't load member {id}: {err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_body_has_exactly_the_contract_fields() {
        let member = NewsletterMember {
            member_number: 834.into(),
            email: "jan@example.com".to_string(),
            name: "Jan Novák".to_string(),
            city: None,
            lang: "cs".to_string(),
            workplace_newsletter_id: Some(13),
        };

        let body = put_body(&member).unwrap();
        let mut keys: Vec<&str> = body
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();

        assert_eq!(
            keys,
            ["city", "email", "lang", "name", "workplace_newsletter_id"]
        );
    }
}
