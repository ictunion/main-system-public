use reqwest::Client;
use rocket::serde::json;
use serde::{Deserialize, Serialize};

use crate::config::Config;

use crate::api::applications::ApplicationDetail;
use crate::data::{Id, Member, MemberNumber};

use crate::api::ApiError;

use log::{debug, error, info};

mod query;

#[derive(Deserialize)]
struct ListmonkResponseData {
    id: i32,
}

#[derive(Deserialize)]
struct ListmonkResponse {
    data: ListmonkResponseData,
}

#[derive(Deserialize)]
struct ListmonkSubscriberRecord {
    id: i32,
}

#[derive(Deserialize)]
struct ListmonkSubscribersResponseData {
    results: Vec<ListmonkSubscriberRecord>,
}

#[derive(Deserialize)]
struct ListmonkSubscribersResponse {
    data: ListmonkSubscribersResponseData,
}

#[derive(Serialize)]
struct ListMonkAtrribs {
    lang: String,
    member_number: MemberNumber, //memberNumber should be serializable
}

#[derive(Serialize)]
struct ListMonkSubscribe {
    email: String,
    name: String,
    status: ListMonkStatus,
    lists: Vec<u32>,
    attribs: ListMonkAtrribs,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ListMonkStatus {
    Enabled,
    Disabled,
}

pub(crate) struct ListMonkDetail<'a> {
    detail: &'a ApplicationDetail,
    member_number: MemberNumber,
    pub(crate) lists: Vec<u32>,
}

pub(crate) fn set_listmonk_lists(list_monk_detail: &mut ListMonkDetail<'_>) {
    // vec![5] to main List, 4 prague, 3 brno

    let location = list_monk_detail
        .detail
        .city
        .clone()
        .expect("City is required")
        .to_lowercase();

    if location.contains("prague") || location.contains("praha") {
        list_monk_detail.lists = vec![4u32, 5u32];
        return;
    }
    if location.contains("brno") {
        list_monk_detail.lists = vec![3u32, 5u32];
        return;
    }
    // fallback — maybe assign a default list or do nothing
    list_monk_detail.lists = vec![5u32];
}

async fn get_subscriber_id_by_email(
    client: &Client,
    host: &str,
    username: &str,
    password: &str,
    email: &str,
) -> Result<Option<i32>, ApiError> {
    let url: String = format!("{host}/api/subscribers");
    let query = format!("subscribers.email='{}'", email.replace('\'', "''"));

    let res: reqwest::Response = client
        .get(url)
        .basic_auth(username, Some(password))
        .query(&[("query", query.as_str())])
        .send()
        .await?;

    let res_status = res.status();
    let text: String = res.text().await?;

    if !res_status.is_success() {
        error!("Error looking up Listmonk subscriber by email: {text}");
        return Err(ApiError::listmonk_error(&text));
    }

    let parsed: ListmonkSubscribersResponse =
        json::from_str(&text).map_err(|_ignored| ApiError::data_missing("Listmonk.Subscribers"))?;

    Ok(parsed.data.results.into_iter().next().map(|r| r.id))
}

pub(crate) async fn subscribe_to_listmonk(
    application_detail: &ApplicationDetail,
    member_number: MemberNumber,
    config: &Config,
    mut tx: sqlx::Transaction<'_, sqlx::Postgres>,
    member_id: Id<Member>,
) -> Result<(), ApiError> {
    let mut list_monk_detail: ListMonkDetail<'_> = ListMonkDetail {
        detail: application_detail,
        lists: vec![0],
        member_number,
    };

    set_listmonk_lists(&mut list_monk_detail);

    let password = config
        .listmonk_password
        .as_ref()
        .ok_or_else(|| ApiError::config_missing("listmonk_password"))?;

    let username = config
        .listmonk_username
        .as_ref()
        .ok_or_else(|| ApiError::config_missing("listmonk_username"))?;

    let host = config
        .listmonk_host
        .as_ref()
        .ok_or_else(|| ApiError::config_missing("listmonk_host"))?;

    let url: String = format!("{host}/api/subscribers");

    let client = Client::new();

    let first_name = list_monk_detail
        .detail
        .first_name
        .clone()
        .ok_or_else(|| ApiError::data_missing("member.first_name"))?;

    let last_name = list_monk_detail
        .detail
        .last_name
        .clone()
        .ok_or_else(|| ApiError::data_missing("member.last_name"))?;

    let name = format!("{first_name} {last_name}");

    let member_number = list_monk_detail.member_number;

    let lang = list_monk_detail
        .detail
        .language
        .clone()
        .ok_or_else(|| ApiError::data_missing("member.language"))?;

    let attribs = ListMonkAtrribs {
        lang,
        member_number,
    };

    let payload: ListMonkSubscribe = ListMonkSubscribe {
        email: list_monk_detail
            .detail
            .email
            .clone()
            .ok_or_else(|| ApiError::data_missing("member.email"))?,
        name,
        status: ListMonkStatus::Enabled,
        attribs,
        lists: list_monk_detail.lists.clone(),
    };

    let res: reqwest::Response = client
        .post(url)
        .header("Content-Type", "application/json")
        .basic_auth(username, Some(password))
        .json(&payload)
        .send()
        .await?;
    let res_status = res.status();
    let text: String = res.text().await?;

    debug!("Status: {res_status}");

    let listmonk_id: i32 = if res_status.as_u16() == 409 {
        info!(
            "Listmonk subscriber already exists for {}, looking up by email",
            payload.email
        );
        get_subscriber_id_by_email(&client, host, username, password, &payload.email)
            .await?
            .ok_or_else(|| ApiError::listmonk_error(&text))?
    } else if !res_status.is_success() {
        error!("Error subscribing user to Listmonk: {text}");
        return Err(ApiError::listmonk_error(&text));
    } else {
        json::from_str::<ListmonkResponse>(&text)
            .map(|r| r.data.id)
            .map_err(|_ignored| ApiError::data_missing("Listmonk.Id"))?
    };

    let status = "enabled";

    let lists = list_monk_detail.lists;
    for list_value in lists {
        let unsingned_list_value = i64::from(list_value);
        query::create_email_subscription(
            &mut *tx,
            member_id,
            unsingned_list_value,
            status,
            listmonk_id,
        )
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
