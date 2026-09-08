//! Imported definitions somebody has read and taken responsibility for.
//!
//! An importer decides what a definition *could* become. This module decides
//! what one is allowed to be. The difference matters most for setup fields:
//! `caprover::setup_variable` marks every field sensitive, because an upstream
//! catalogue has no way to say which of its variables is a credential, and
//! masking a time zone is a smaller mistake than showing an API key. A review
//! is where somebody who read the app says which is which.
//!
//! Nothing here is offered to anyone. A reviewed template is a template a
//! person could be shown; whether it appears in the catalog is a separate
//! decision, made by the project owner, and this module does not make it.
use crate::setup::PlanTemplate;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Where a definition came from, precisely enough to fetch it again.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplateOrigin {
    /// Which importer maps this format. The only mapping path there is.
    pub importer: String,
    pub repository: String,
    /// The commit the definition was taken from, not a branch.
    pub revision: String,
    pub path: String,
    /// The upstream project's licence, which governs reuse of the definition.
    pub license: String,
}

/// What a review concluded. A review that can only say yes is not a review,
/// so the outcome is recorded either way and the reason travels with it.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplatePromotion {
    /// `approved` or `withheld`. Only an approved template may be offered.
    pub state: String,
    /// Why, in terms somebody could disagree with.
    pub reason: String,
}

/// One image the plan runs, and what was checked about it.
///
/// A recipe audits a single image. A multi-service template cannot: it is only
/// as portable as its least portable image and only as current as its oldest,
/// so each one is recorded separately.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplateImageAudit {
    pub image: String,
    pub source_url: String,
    pub checked_at: String,
    /// When the registry last rebuilt this tag. An old date is not a fault by
    /// itself, but it is the fact a promotion decision turns on most often.
    pub last_updated: String,
    pub container_platforms: Vec<String>,
    /// Per-architecture digests. These registries publish no index digest for
    /// a tag, so pinning each architecture is what can actually be verified.
    pub digests: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplateRequirements {
    pub docker_engine_os: String,
    pub compose_major: u8,
    pub local_storage_required: bool,
    pub images: Vec<TemplateImageAudit>,
}

/// What a review decided about one setup field.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FieldReview {
    /// What to call it, in this project's words rather than upstream's.
    pub label: String,
    /// Whether this answer is a credential. The importer says yes to
    /// everything; saying no here is a decision somebody made after reading
    /// what the app does with the value.
    pub sensitive: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReviewedTemplate {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    pub catalog_name: String,
    pub description: String,
    pub category: String,
    /// The application's own licence, which is not the definition's.
    pub license: String,
    pub source_url: String,
    pub documentation_url: String,
    pub verified_at: String,
    /// The recorded run that proves this template installs and survives a
    /// reinstall. A template without one has not been verified, whatever else
    /// it declares.
    pub lifecycle_proof: String,
    pub origin: TemplateOrigin,
    pub requirements: TemplateRequirements,
    pub promotion: TemplatePromotion,
    pub data_storage: String,
    pub risk_notes: Vec<String>,
    /// One entry per setup field the import produces. Both directions are
    /// checked, so a field upstream adds cannot arrive unreviewed and a review
    /// left behind by a removed field cannot sit unnoticed.
    pub fields: BTreeMap<String, FieldReview>,
    /// The upstream definition, normalized to JSON exactly as the import
    /// report normalizes all of them. Kept verbatim so the mapping can be
    /// reproduced and audited without the archive.
    pub definition: String,
}

impl ReviewedTemplate {
    /// Whether this template may be offered to anyone.
    ///
    /// Passing a review is not the same as being worth installing. A template
    /// can resolve cleanly, install, and still be one nobody should be handed
    /// — an image whose registry stopped rebuilding it years ago, say. This is
    /// the difference, and it is the project owner's decision to change.
    pub fn offerable(&self) -> bool {
        self.promotion.state == "approved"
    }

    /// The template this review permits, or why it does not permit one.
    pub fn plan_template(&self) -> Result<PlanTemplate, String> {
        if self.origin.importer != "caprover" {
            return Err(format!(
                "no importer named {:?} to map this definition",
                self.origin.importer
            ));
        }
        let outcome = crate::importers::caprover::import(&self.id, &self.definition)
            .map_err(|reason| format!("{}: {reason}", self.id))?;
        let mut template = outcome.template.ok_or_else(|| {
            let named: Vec<&str> = outcome
                .limitations
                .iter()
                .map(|limit| limit.feature())
                .collect();
            format!(
                "{}: the definition is no longer importable ({})",
                self.id,
                named.join(", ")
            )
        })?;

        for field in &mut template.fields {
            let review = self.fields.get(&field.key).ok_or_else(|| {
                format!(
                    "{}: setup field {:?} has not been reviewed",
                    self.id, field.key
                )
            })?;
            field.label = review.label.clone();
            field.sensitive = review.sensitive;
        }
        for key in self.fields.keys() {
            if !template.fields.iter().any(|field| &field.key == key) {
                return Err(format!(
                    "{}: review names setup field {key:?}, which this definition no longer declares",
                    self.id
                ));
            }
        }
        // Every image that would run has to have been looked at, and every
        // image looked at has to still be one that runs.
        for service in &template.plan.services {
            if !self
                .requirements
                .images
                .iter()
                .any(|audit| audit.image == service.image)
            {
                return Err(format!(
                    "{}: image {:?} has not been audited",
                    self.id, service.image
                ));
            }
        }
        for audit in &self.requirements.images {
            if !template
                .plan
                .services
                .iter()
                .any(|service| service.image == audit.image)
            {
                return Err(format!(
                    "{}: audit names image {:?}, which this definition no longer runs",
                    self.id, audit.image
                ));
            }
        }

        template
            .validate()
            .map_err(|reason| format!("{}: {reason}", self.id))?;
        Ok(template)
    }
}

const CODIMD: &str = include_str!("codimd.json");

/// Every template a review has passed. Being here is not being offered.
pub fn reviewed_templates() -> Vec<ReviewedTemplate> {
    [CODIMD]
        .into_iter()
        .map(|source| serde_json::from_str(source).expect("bundled reviewed templates must parse"))
        .collect()
}

pub fn reviewed_template(id: &str) -> Option<ReviewedTemplate> {
    reviewed_templates()
        .into_iter()
        .find(|template| template.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reviewed_template_resolves_to_a_valid_plan() {
        let templates = reviewed_templates();
        assert!(!templates.is_empty(), "the allowlist is empty");
        for reviewed in templates {
            let template = reviewed
                .plan_template()
                .unwrap_or_else(|reason| panic!("{reason}"));
            assert!(
                !template.plan.services.is_empty(),
                "{} deploys nothing",
                reviewed.id
            );
            // A reviewed template has to render, not merely construct.
            let compose = template.plan.to_compose().expect("plan should render");
            assert!(!compose.contains("$$"), "{}: {compose}", reviewed.id);

            // Everything a recipe must prove about itself.
            assert_eq!(reviewed.schema_version, 1);
            assert!(!reviewed.risk_notes.is_empty(), "{}", reviewed.id);
            // A named proof that is not there is worse than none: it reads
            // as verification nobody can check.
            let proof =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(&reviewed.lifecycle_proof);
            assert!(
                reviewed.lifecycle_proof.starts_with("docs/evidence/") && proof.is_file(),
                "{} names a lifecycle proof that is not there: {}",
                reviewed.id,
                reviewed.lifecycle_proof
            );
            assert!(
                reviewed.origin.revision.len() == 40
                    && reviewed
                        .origin
                        .revision
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit()),
                "{} pins no upstream commit",
                reviewed.id
            );
            assert!(
                !reviewed.requirements.images.is_empty(),
                "{} audits no image",
                reviewed.id
            );
            for audit in &reviewed.requirements.images {
                assert!(
                    !audit.container_platforms.is_empty(),
                    "{}: {} claims no platform",
                    reviewed.id,
                    audit.image
                );
                assert_eq!(
                    audit.digests.keys().cloned().collect::<Vec<_>>(),
                    audit.container_platforms,
                    "{}: {} pins a different set of platforms than it claims",
                    reviewed.id,
                    audit.image
                );
                for digest in audit.digests.values() {
                    assert!(digest.starts_with("sha256:"), "{}", reviewed.id);
                }
            }
            assert!(
                matches!(reviewed.promotion.state.as_str(), "approved" | "withheld"),
                "{} records no promotion decision",
                reviewed.id
            );
            assert!(
                reviewed.promotion.reason.len() > 20,
                "{} gives no reason for its promotion decision",
                reviewed.id
            );
            for service in &template.plan.services {
                let tag = service.image.rsplit_once(':').map(|(_, tag)| tag);
                assert!(
                    tag.is_some_and(|tag| !matches!(tag, "latest" | "stable" | "main")),
                    "{} runs an unpinned image",
                    reviewed.id
                );
            }
        }
    }

    #[test]
    fn a_review_decides_which_answers_are_credentials() {
        let reviewed = reviewed_template("codimd").expect("codimd is reviewed");
        let template = reviewed.plan_template().unwrap();
        let review = template.setup_review().unwrap();

        // The importer marks every field sensitive because upstream cannot
        // say. Without a review this would be a masked box with its default
        // withheld, which is the wrong way to ask for a time zone.
        let zone = review
            .fields
            .iter()
            .find(|field| field.key == "CAP_TIMEZONE")
            .expect("codimd asks for a time zone");
        assert_eq!(zone.control, "text");
        assert!(!zone.sensitive);
        assert_eq!(zone.default.as_deref(), Some("Europe/London"));
        assert_eq!(zone.label, "Time zone");

        // The generated credential is counted, never asked for or shown.
        assert_eq!(review.generated_credential_count, 1);
        assert!(!review.fields.iter().any(|field| field.key.contains("PASS")));
    }

    /// Nothing is offered yet, and that is the current state of the project
    /// rather than an oversight. When this starts failing, somebody approved a
    /// template, which is a decision that should be visible in a diff.
    #[test]
    fn no_reviewed_template_is_offerable_without_an_explicit_approval() {
        let offerable: Vec<String> = reviewed_templates()
            .into_iter()
            .filter(ReviewedTemplate::offerable)
            .map(|template| template.id)
            .collect();
        assert!(
            offerable.is_empty(),
            "these templates are marked approved: {offerable:?}"
        );
    }

    #[test]
    fn an_image_nobody_audited_stops_the_template() {
        let mut reviewed = reviewed_template("codimd").unwrap();
        reviewed.requirements.images.remove(0);
        let error = reviewed.plan_template().unwrap_err();
        assert!(error.contains("has not been audited"), "{error}");
    }

    #[test]
    fn a_field_nobody_reviewed_stops_the_template_rather_than_shipping_masked() {
        let mut reviewed = reviewed_template("codimd").unwrap();
        reviewed.fields.remove("CAP_TIMEZONE");
        let error = reviewed.plan_template().unwrap_err();
        assert!(error.contains("has not been reviewed"), "{error}");

        // And a review left behind by a field upstream removed is caught too,
        // because it means the review was written against a different app.
        let mut stale = reviewed_template("codimd").unwrap();
        stale.fields.insert(
            "CAP_GONE".into(),
            FieldReview {
                label: "Gone".into(),
                sensitive: false,
            },
        );
        let error = stale.plan_template().unwrap_err();
        assert!(error.contains("no longer declares"), "{error}");
    }
}
