//! Bilingual (Czech / English) documents generated for a workplace.
//! All of them list the workplace's representatives and are signed by the union chairperson.
use chrono::NaiveDate;
use log::info;
use tokio::fs;
use uuid::Uuid;

use super::{ProcessingError, query, render_tex};
use crate::config::Config;
use crate::data::{Id, Workplace};
use crate::db::DbPool;
use crate::media::TexEscape;

#[derive(Debug)]
pub enum WorkplaceDocument {
    /// Entrustment of workplace executive committee
    /// to negotiate with the employer on behalf of the union
    Entrustment {
        employer: EmployerInput,
        /// Date of the workplace members' meeting which elected the committee
        meeting_date: NaiveDate,
        /// Members who are no longer on the committee as of the meeting date.
        /// When not empty the entrustment gets an extra section listing them.
        removed_members: Vec<Uuid>,
    },
    /// Confirmation that representatives are members of the union.
    /// Short name isn't printed, only used for the file name.
    MemberConfirmation { workplace_short_name: String },
    /// Notification to the employer that the union operates at the workplace (§ 286 zákoníku práce)
    WorkplaceAnnouncement(EmployerInput),
}

impl WorkplaceDocument {
    /// Used both as name of the LaTeX template and name of the stored file
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::Entrustment { .. } => "entrustment",
            Self::MemberConfirmation { .. } => "member_confirmation",
            Self::WorkplaceAnnouncement(_) => "workplace_announcement",
        }
    }

    fn template(&self) -> &'static str {
        match self {
            Self::Entrustment { .. } => include_str!("../../latex/entrustment.tex"),
            Self::MemberConfirmation { .. } => include_str!("../../latex/member_confirmation.tex"),
            Self::WorkplaceAnnouncement(_) => {
                include_str!("../../latex/workplace_announcement.tex")
            }
        }
    }

    /// Short name of the workplace as entered by whoever requested the document
    fn workplace_short_name(&self) -> &str {
        match self {
            Self::Entrustment { employer, .. } | Self::WorkplaceAnnouncement(employer) => {
                &employer.workplace_short_name
            }
            Self::MemberConfirmation {
                workplace_short_name,
            } => workplace_short_name,
        }
    }
}

/// Employer details which are not stored in the database (yet)
/// and have to be supplied by whoever requests the document.
/// Company name is the workplace name from the database.
#[derive(Debug)]
pub struct EmployerInput {
    /// Short name of workplace used in union context (e.g. "ACME CZ")
    pub workplace_short_name: String,
    pub company_address: String,
    /// Company identification number (IČ)
    pub company_id: String,
}

#[derive(Debug)]
pub struct Representative {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
}

pub(super) async fn process_new_document(
    workplace_id: Id<Workplace>,
    document: &WorkplaceDocument,
    config: &Config,
    db_pool: &DbPool,
) -> Result<(), ProcessingError> {
    let name = document.name();

    let chairperson_id = config
        .union_chairperson_member_id
        .ok_or(ProcessingError::MissingChairpersonMemberId)?;
    let chairperson_name = query::get_member_full_name(db_pool, chairperson_id.into()).await?;

    let representatives = query::list_workplace_representatives(db_pool, workplace_id).await?;
    if representatives.is_empty() {
        return Err(ProcessingError::NoRepresentatives(workplace_id));
    }

    let mut tex_header = print_common_tex_header(&representatives, &chairperson_name);
    match document {
        WorkplaceDocument::Entrustment {
            employer,
            meeting_date,
            removed_members,
        } => {
            let company_name = query::get_workplace_name(db_pool, workplace_id).await?;
            tex_header.push_str(&print_employer_tex_header(employer, &company_name));
            tex_header.push_str(&print_meeting_date_tex_header(*meeting_date));

            if !removed_members.is_empty() {
                let removed = query::list_members_by_ids(db_pool, removed_members).await?;
                tex_header.push_str(&print_removed_members_tex_header(&removed));
            }
        }
        WorkplaceDocument::MemberConfirmation { .. } => {}
        WorkplaceDocument::WorkplaceAnnouncement(employer) => {
            let chairperson_email = config
                .union_chairperson_email
                .as_deref()
                .ok_or(ProcessingError::MissingChairpersonEmail)?;
            let company_name = query::get_workplace_name(db_pool, workplace_id).await?;
            tex_header.push_str(&print_employer_tex_header(employer, &company_name));
            tex_header.push_str(&print_chairperson_email_tex_header(chairperson_email));
        }
    }

    let processing_dir = format!("data/workplace_documents/{}", Uuid::new_v4());
    info!("Printing {name} pdf for workplace {workplace_id} at {processing_dir}");
    fs::create_dir_all(&processing_dir).await?;

    fs::write(format!("{processing_dir}/data.tex"), tex_header).await?;
    fs::write(format!("{processing_dir}/{name}.tex"), document.template()).await?;

    let pdf_data = render_tex(config, &processing_dir, name).await?;
    let file_name = file_name(name, document.workplace_short_name());
    query::insert_workplace_pdf(db_pool, workplace_id, &file_name, &pdf_data).await?;
    info!("{name} for workplace {workplace_id} stored");

    fs::remove_dir_all(processing_dir).await?;

    Ok(())
}

/// Stored file name: simplified workplace short name followed by document type,
/// e.g. `acme-cz_entrustment`
fn file_name(document_name: &str, workplace_short_name: &str) -> String {
    let slug = slugify(workplace_short_name);
    if slug.is_empty() {
        document_name.to_string()
    } else {
        format!("{slug}_{document_name}")
    }
}

/// Lowercase ASCII letters and digits separated by single dashes.
/// Czech diacritics are mapped to their base letter, other characters become separators.
fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for c in text.to_lowercase().chars() {
        let c = match c {
            'á' | 'ä' => 'a',
            'č' => 'c',
            'ď' => 'd',
            'é' | 'ě' => 'e',
            'í' => 'i',
            'ľ' | 'ĺ' => 'l',
            'ň' => 'n',
            'ó' | 'ö' | 'ô' => 'o',
            'ř' | 'ŕ' => 'r',
            'š' => 's',
            'ť' => 't',
            'ú' | 'ů' | 'ü' => 'u',
            'ý' => 'y',
            'ž' => 'z',
            other => other,
        };
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
}

// Dates are not arbitrary strings so they don't need escaping.
// Parts are joined by non-breaking spaces so a date never wraps.

fn format_date_cs(date: NaiveDate) -> String {
    date.format("%-d.~%-m.~%Y").to_string()
}

fn format_date_en(date: NaiveDate) -> String {
    date.format("%-d~%B~%Y").to_string()
}

/// One representative per line, followed by date of birth (if known)
fn print_representatives(
    representatives: &[Representative],
    born_label: &str,
    format_date: fn(NaiveDate) -> String,
) -> String {
    representatives
        .iter()
        .map(|rep| match rep.date_of_birth {
            Some(date) => format!(
                "{}, \\mbox{{{born_label}~{}}}",
                full_name(rep),
                format_date(date)
            ),
            None => full_name(rep),
        })
        .collect::<Vec<_>>()
        .join("\\\\\n")
}

fn full_name(rep: &Representative) -> String {
    format!(
        "{} {}",
        rep.first_name.as_deref().unwrap_or(""),
        rep.last_name.as_deref().unwrap_or("")
    )
    .trim()
    .escape_tex()
}

/// Names only, one per line
fn print_representative_names(representatives: &[Representative]) -> String {
    representatives
        .iter()
        .map(full_name)
        .collect::<Vec<_>>()
        .join("\\\\\n")
}

/// Definitions every workplace document uses
fn print_common_tex_header(representatives: &[Representative], chairperson_name: &str) -> String {
    format!(
        "\
\\def\\RepresentativesCs{{{}}}
\\def\\RepresentativesEn{{{}}}
\\def\\RepresentativeNames{{{}}}
\\def\\ChairpersonName{{{}}}
",
        print_representatives(representatives, "nar.", format_date_cs),
        print_representatives(representatives, "born", format_date_en),
        print_representative_names(representatives),
        chairperson_name.escape_tex(),
    )
}

fn print_employer_tex_header(employer: &EmployerInput, company_name: &str) -> String {
    format!(
        "\
\\def\\WorkplaceName{{{}}}
\\def\\CompanyName{{{}}}
\\def\\CompanyAddress{{{}}}
\\def\\CompanyId{{{}}}
",
        employer.workplace_short_name.escape_tex(),
        company_name.escape_tex(),
        employer.company_address.escape_tex(),
        employer.company_id.escape_tex(),
    )
}

fn print_chairperson_email_tex_header(email: &str) -> String {
    format!("\\def\\ChairpersonEmail{{{}}}\n", email.escape_tex())
}

/// Switches on the removed members section of the entrustment
fn print_removed_members_tex_header(removed: &[Representative]) -> String {
    format!(
        "\
\\RemovedMemberstrue
\\def\\RemovedMembersCs{{{}}}
\\def\\RemovedMembersEn{{{}}}
",
        print_representatives(removed, "nar.", format_date_cs),
        print_representatives(removed, "born", format_date_en),
    )
}

fn print_meeting_date_tex_header(meeting_date: NaiveDate) -> String {
    format!(
        "\
\\def\\MeetingDateCs{{{}}}
\\def\\MeetingDateEn{{{}}}
",
        format_date_cs(meeting_date),
        format_date_en(meeting_date),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rep(first: &str, last: &str, dob: Option<NaiveDate>) -> Representative {
        Representative {
            first_name: Some(first.to_string()),
            last_name: Some(last.to_string()),
            date_of_birth: dob,
        }
    }

    #[test]
    fn removed_members_header_is_plain_tex_lines() {
        let removed = [rep(
            "Hana",
            "Pokorová",
            NaiveDate::from_ymd_opt(1998, 2, 28),
        )];
        assert_eq!(
            print_removed_members_tex_header(&removed),
            "\\RemovedMemberstrue\n\
\\def\\RemovedMembersCs{Hana Pokorová, \\mbox{nar.~28.~2.~1998}}\n\
\\def\\RemovedMembersEn{Hana Pokorová, \\mbox{born~28~February~1998}}\n"
        );
    }

    #[test]
    fn slugifies_workplace_name() {
        assert_eq!(
            slugify("Third Testing Workplace"),
            "third-testing-workplace"
        );
        assert_eq!(slugify("  Žluťoučký kůň, s.r.o. "), "zlutoucky-kun-s-r-o");
        assert_eq!(slugify("***"), "");
    }

    #[test]
    fn file_name_includes_workplace() {
        assert_eq!(file_name("entrustment", "ACME CZ"), "acme-cz_entrustment");
        assert_eq!(file_name("entrustment", "***"), "entrustment");
    }

    #[test]
    fn formats_dates() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        assert_eq!(format_date_cs(date), "5.~3.~2026");
        assert_eq!(format_date_en(date), "5~March~2026");
    }

    #[test]
    fn prints_representatives_one_per_line() {
        let reps = [
            rep("Jana", "Nováková", NaiveDate::from_ymd_opt(1990, 1, 1)),
            rep("Karel", "Dvořák & syn", None),
        ];
        assert_eq!(
            print_representatives(&reps, "nar.", format_date_cs),
            "Jana Nováková, \\mbox{nar.~1.~1.~1990}\\\\\nKarel Dvořák \\& syn"
        );
    }

    #[test]
    fn prints_representative_names_one_per_line() {
        let reps = [
            rep("Jana", "Nováková", NaiveDate::from_ymd_opt(1990, 1, 1)),
            rep("Karel", "Dvořák & syn", None),
        ];
        assert_eq!(
            print_representative_names(&reps),
            "Jana Nováková\\\\\nKarel Dvořák \\& syn"
        );
    }
}
