//! Keeps Melon Head's test fixtures (`melon-head/test/fixtures/*.json`) in step
//! with what Orca sends. Each fixture is read into the response struct and
//! serialized back; the result must be the same JSON. A renamed, added, removed
//! or retyped field fails here, so the fixture is updated in the same change and
//! the frontend's decoder tests then run against the new shape.
//!
//! The structs derive `Deserialize` only under `cfg(test)`, for this test.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

use super::applications::{
    ApplicationAcceptedSummary, ApplicationDetail, ApplicationInvalidSummary,
    ApplicationProcessingSummary, ApplicationRejectedSummary, ApplicationSummary,
    ApplicationUnverifiedSummary, ApplicationWorkplace,
};
use super::files::FileInfo;
use super::members::{MemberDetail, MemberOffice, MemberSummary, Occupation};
use super::notes::{NoteDetail, NoteSummary};
use super::session::SessionInfo;
use super::stats::{ApplicationsBasicStats, MembersBasicStats, WorkplacesBasicStats};
use super::workplaces::{WorkplaceMemberSummary, WorkplaceSummary};
use crate::server::oid::User;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../melon-head/test/fixtures")
}

type Check = fn(&str) -> Result<(), String>;

fn round_trip<T: DeserializeOwned + Serialize>(name: &str) -> Result<(), String> {
    let path = fixtures_dir().join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{name}: {e}"))?;
    let fixture: Value = serde_json::from_str(&text).map_err(|e| format!("{name}: {e}"))?;
    let parsed: T = serde_json::from_value(fixture.clone())
        .map_err(|e| format!("{name}: Orca can't read the fixture: {e}"))?;
    let sent = serde_json::to_value(&parsed).expect("response structs serialize");
    if sent == fixture {
        Ok(())
    } else {
        Err(format!(
            "{name}: Orca sends\n{}\nbut the fixture has\n{}",
            serde_json::to_string_pretty(&sent).unwrap(),
            serde_json::to_string_pretty(&fixture).unwrap()
        ))
    }
}

/// Fixture name -> the struct Orca serializes for it.
const CHECKS: &[(&str, Check)] = &[
    ("member_summary", round_trip::<MemberSummary>),
    ("member_detail", round_trip::<MemberDetail>),
    ("member_occupation", round_trip::<Occupation>),
    ("member_office", round_trip::<MemberOffice>),
    ("application_summary", round_trip::<ApplicationSummary>),
    (
        "application_unverified_summary",
        round_trip::<ApplicationUnverifiedSummary>,
    ),
    (
        "application_accepted_summary",
        round_trip::<ApplicationAcceptedSummary>,
    ),
    (
        "application_rejected_summary",
        round_trip::<ApplicationRejectedSummary>,
    ),
    (
        "application_processing_summary",
        round_trip::<ApplicationProcessingSummary>,
    ),
    (
        "application_invalid_summary",
        round_trip::<ApplicationInvalidSummary>,
    ),
    ("application_detail", round_trip::<ApplicationDetail>),
    ("application_workplace", round_trip::<ApplicationWorkplace>),
    ("workplace_summary", round_trip::<WorkplaceSummary>),
    ("workplace_member", round_trip::<WorkplaceMemberSummary>),
    ("note_detail", round_trip::<NoteDetail>),
    ("note_summary", round_trip::<NoteSummary>),
    ("stats_applications", round_trip::<ApplicationsBasicStats>),
    ("stats_members", round_trip::<MembersBasicStats>),
    ("stats_workplaces", round_trip::<WorkplacesBasicStats>),
    ("session", round_trip::<SessionInfo>),
    ("user", round_trip::<User>),
    ("file_info", round_trip::<FileInfo>),
];

#[test]
#[ignore = "reads ../melon-head, outside the nix build source"]
fn fixtures_match_what_orca_sends() {
    let errors: Vec<String> = CHECKS
        .iter()
        .filter_map(|(name, check)| check(name).err())
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n\n"));
}

/// A new fixture must be added to `CHECKS`. `payment_*` ones come from the bank
/// API, not Orca, and can't be checked here.
#[test]
#[ignore = "reads ../melon-head, outside the nix build source"]
fn every_orca_fixture_is_checked() {
    let on_disk: BTreeSet<String> = std::fs::read_dir(fixtures_dir())
        .expect("melon-head/test/fixtures exists")
        .filter_map(|e| {
            e.ok()?
                .file_name()
                .to_str()?
                .strip_suffix(".json")
                .map(String::from)
        })
        .filter(|name| !name.starts_with("payment_"))
        .collect();
    let checked: BTreeSet<String> = CHECKS.iter().map(|(n, _)| n.to_string()).collect();
    assert_eq!(on_disk, checked);
}
