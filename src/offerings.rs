//! What a person can actually be offered to install.
//!
//! There are two kinds of reviewed thing in this project. A *recipe* carries
//! its own Compose file and installs from it byte for byte. A *reviewed
//! template* is an upstream definition somebody read, audited and mapped
//! through an importer. Until now only recipes reached anyone: the template
//! allowlist was a shelf that nothing outside its own tests read, so
//! `ReviewedTemplate::offerable` could be true without meaning anything.
//!
//! This module is the one place that answers "what can be installed, and what
//! is it". Both the review a person is shown and the install that follows
//! resolve through [`offering`], so the two cannot describe different things —
//! the same reason `template_for` was made a single function when the setup
//! form was built.
use crate::error::{AppError, AppResult};
use crate::recipes::Recipe;
use crate::setup::PlanTemplate;
use crate::templates::ReviewedTemplate;
use serde::Serialize;

/// One installable thing, and which of the two reviewed sources it came from.
#[derive(Debug, Clone)]
pub enum Offering {
    Recipe(Box<Recipe>),
    Template(Box<ReviewedTemplate>),
}

/// What an install review shows, in one shape whichever source it came from.
///
/// The field names match what the launcher already reads off a recipe, so the
/// review dialog did not have to learn a second vocabulary to describe an
/// imported app.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OfferingSummary {
    pub id: String,
    pub display_name: String,
    pub catalog_name: String,
    pub description: String,
    pub category: String,
    pub license: String,
    pub version: String,
    /// The image behind the address a person opens. A template may run more
    /// than one container; `service_count` on the setup review says how many,
    /// and the risk notes say what they are.
    pub image: String,
    pub source_url: String,
    pub documentation_url: String,
    pub verified_at: String,
    pub launch_url: String,
    pub host_port: u16,
    pub container_port: u16,
    pub data_storage: String,
    pub risk_notes: Vec<String>,
}

/// Every recipe, plus every reviewed template a review approved.
///
/// A withheld template is deliberately absent rather than present-and-flagged:
/// nothing downstream should have to remember to check, and a filter that is
/// applied once here cannot be forgotten at a second call site.
pub fn offerings() -> Vec<Offering> {
    let mut offerings: Vec<Offering> = crate::recipes::reviewed_recipes()
        .into_iter()
        .map(|recipe| Offering::Recipe(Box::new(recipe)))
        .collect();
    offerings.extend(
        crate::templates::reviewed_templates()
            .into_iter()
            .filter(ReviewedTemplate::offerable)
            .map(|template| Offering::Template(Box::new(template))),
    );
    offerings
}

pub fn offering(id: &str) -> Option<Offering> {
    offerings().into_iter().find(|offering| offering.id() == id)
}

impl Offering {
    pub fn id(&self) -> &str {
        match self {
            Offering::Recipe(recipe) => &recipe.id,
            Offering::Template(template) => &template.id,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Offering::Recipe(recipe) => &recipe.display_name,
            Offering::Template(template) => &template.display_name,
        }
    }

    /// The name this app is known by in the catalog, which is what gives an
    /// installed app its icon.
    pub fn catalog_name(&self) -> &str {
        match self {
            Offering::Recipe(recipe) => &recipe.catalog_name,
            Offering::Template(template) => &template.catalog_name,
        }
    }

    /// Whether this offering installs from its own reviewed Compose file.
    ///
    /// A recipe does, and rendering it from a plan instead would put a second
    /// author between the review and what ships. A template has no Compose
    /// file of its own; rendering is the only way it exists.
    pub fn is_recipe(&self) -> bool {
        matches!(self, Offering::Recipe(_))
    }

    /// The reviewed recipe behind this offering, on the chosen port, or
    /// `None` because it is an imported template instead.
    ///
    /// A recipe remaps and revalidates its own Compose text, so an address the
    /// rewrite left behind is refused here rather than at the daemon. This is
    /// what lets the install path keep using a recipe's own Compose file byte
    /// for byte instead of re-rendering it from a plan.
    pub fn recipe(&self, host_port: Option<u16>) -> AppResult<Option<Recipe>> {
        match self {
            Offering::Recipe(recipe) => Ok(Some(match host_port {
                Some(port) => recipe.with_host_port(port).map_err(AppError::invalid)?,
                None => (**recipe).clone(),
            })),
            Offering::Template(_) => Ok(None),
        }
    }

    /// The template this offering installs from.
    ///
    /// For a recipe this is the field-less template the setup review has always
    /// projected. For an imported app it is the reviewed mapping, which is
    /// where its typed fields and generated credentials come from.
    pub fn plan_template(&self, host_port: Option<u16>) -> AppResult<PlanTemplate> {
        match self {
            Offering::Recipe(recipe) => {
                let recipe = match host_port {
                    Some(port) => recipe.with_host_port(port).map_err(AppError::invalid)?,
                    None => (**recipe).clone(),
                };
                Ok(PlanTemplate {
                    first_start: None,
                    seeds: Vec::new(),
                    plan: crate::plan::plan_for_recipe(&recipe).map_err(AppError::invalid)?,
                    fields: Vec::new(),
                    secrets: Vec::new(),
                })
            }
            Offering::Template(template) => {
                let mut mapped = template.plan_template().map_err(AppError::invalid)?;
                if let Some(port) = host_port {
                    check_host_port(port)?;
                    // A preference, not a promise: the installer steps past a
                    // busy port, and reuses the recorded one on a reinstall so
                    // a saved link keeps working.
                    mapped.plan.set_published_host(port);
                }
                Ok(mapped)
            }
        }
    }

    /// Everything the install review shows, for the port this would install on.
    pub fn summary(&self, host_port: Option<u16>) -> AppResult<OfferingSummary> {
        match self {
            Offering::Recipe(recipe) => {
                let recipe = match host_port {
                    Some(port) => recipe.with_host_port(port).map_err(AppError::invalid)?,
                    None => (**recipe).clone(),
                };
                Ok(OfferingSummary {
                    id: recipe.id.clone(),
                    display_name: recipe.display_name.clone(),
                    catalog_name: recipe.catalog_name.clone(),
                    description: recipe.description.clone(),
                    category: recipe.category.clone(),
                    license: recipe.license.clone(),
                    version: recipe.version.clone(),
                    image: recipe.image.clone(),
                    source_url: recipe.source_url.clone(),
                    documentation_url: recipe.documentation_url.clone(),
                    verified_at: recipe.verified_at.clone(),
                    launch_url: recipe.launch_url.clone(),
                    host_port: recipe.host_port,
                    container_port: recipe.container_port,
                    data_storage: recipe.data_storage.clone(),
                    risk_notes: recipe.risk_notes.clone(),
                })
            }
            Offering::Template(template) => {
                let mapped = self.plan_template(host_port)?;
                let (service, port) = mapped.plan.published().ok_or_else(|| {
                    AppError::invalid(format!("{} publishes no address to open.", template.id))
                })?;
                // The version a reviewed definition names, and otherwise the
                // tag the published image is pinned to. They agree for an app
                // that is one service; for Dify, whose front door is an
                // nginx, the tag is nginx's version and the definition's is
                // Dify's.
                let version = template.version.clone().unwrap_or_else(|| {
                    service
                        .image
                        .rsplit_once(':')
                        .map(|(_, tag)| tag.to_owned())
                        .unwrap_or_default()
                });
                Ok(OfferingSummary {
                    id: template.id.clone(),
                    display_name: template.display_name.clone(),
                    catalog_name: template.catalog_name.clone(),
                    description: template.description.clone(),
                    category: template.category.clone(),
                    license: template.license.clone(),
                    version,
                    image: service.image.clone(),
                    source_url: template.source_url.clone(),
                    documentation_url: template.documentation_url.clone(),
                    verified_at: template.verified_at.clone(),
                    // The same address `install_template_with` writes into the
                    // registry, built the same way, so the review and the
                    // installed app cannot disagree about where the app lives.
                    launch_url: format!("http://localhost:{}", port.host),
                    host_port: port.host,
                    container_port: port.container,
                    data_storage: template.data_storage.clone(),
                    risk_notes: template.risk_notes.clone(),
                })
            }
        }
    }
}

/// Ports below 1024 need privileges this app does not ask for, and 0 asks the
/// kernel to choose one nothing downstream could then address. The wording
/// matches `Recipe::with_host_port` so a person sees one message either way.
fn check_host_port(port: u16) -> AppResult<()> {
    if port < 1024 {
        return Err(AppError::invalid("Choose a port between 1024 and 65535."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reviewed_recipe_is_offered_and_summarizes_as_itself() {
        for recipe in crate::recipes::reviewed_recipes() {
            let offering = offering(&recipe.id).expect("a reviewed recipe is offered");
            assert!(offering.is_recipe());
            let summary = offering.summary(None).expect("recipe should summarize");
            assert_eq!(summary.launch_url, recipe.launch_url);
            assert_eq!(summary.host_port, recipe.host_port);
            assert_eq!(summary.image, recipe.image);
            assert_eq!(summary.version, recipe.version);
        }
    }

    /// The whole point of the allowlist: a review that says no keeps the app
    /// away from people, rather than merely recording a preference.
    #[test]
    fn a_withheld_template_is_not_offered_at_all() {
        for template in crate::templates::reviewed_templates() {
            if template.offerable() {
                continue;
            }
            assert!(
                offering(&template.id).is_none(),
                "{} is withheld but reachable",
                template.id
            );
        }
    }

    #[test]
    fn an_approved_template_is_offered_and_describes_its_published_service() {
        let approved: Vec<ReviewedTemplate> = crate::templates::reviewed_templates()
            .into_iter()
            .filter(ReviewedTemplate::offerable)
            .collect();
        for template in approved {
            let offering = offering(&template.id).expect("an approved template is offered");
            assert!(!offering.is_recipe());
            let summary = offering.summary(None).expect("template should summarize");
            assert_eq!(summary.id, template.id);
            assert_eq!(summary.catalog_name, template.catalog_name);
            assert!(!summary.risk_notes.is_empty());

            // The address shown is the address the installer would write, and
            // the container port is the one the image actually listens on.
            assert_eq!(
                summary.launch_url,
                format!("http://localhost:{}", summary.host_port)
            );
            assert!(summary.host_port >= 1024);
            let mapped = offering.plan_template(None).expect("template should map");
            let (service, port) = mapped.plan.published().expect("published endpoint");
            assert_eq!(summary.image, service.image);
            assert_eq!(summary.container_port, port.container);

            // A pinned tag is what the version claim rests on, unless the
            // reviewed definition names the app's own version.
            assert!(!summary.version.is_empty());
            let named = matches!(offering, Offering::Template(template)
                if template.version.as_deref() == Some(summary.version.as_str()));
            assert!(
                named || service.image.ends_with(&format!(":{}", summary.version)),
                "{}: version {} is neither reviewed nor the image's tag",
                summary.id,
                summary.version
            );
        }
    }

    /// A chosen port has to reach the plan, or the address a person was shown
    /// is not the address they get. This is the same failure the platform
    /// placeholders were introduced to prevent, one layer up.
    #[test]
    fn a_chosen_port_moves_the_address_for_either_source() {
        for offering in offerings() {
            let summary = offering.summary(Some(41234)).expect("summary on a port");
            assert_eq!(summary.host_port, 41234);
            assert_eq!(summary.launch_url, "http://localhost:41234");
            let mapped = offering
                .plan_template(Some(41234))
                .expect("template on a port");
            assert_eq!(mapped.plan.published().expect("published").1.host, 41234);
        }
    }

    #[test]
    fn a_privileged_port_is_refused_for_either_source() {
        for offering in offerings() {
            let error = offering
                .summary(Some(80))
                .expect_err("a privileged port must be refused");
            assert!(error.message.contains("1024"), "{}", error.message);
            assert!(offering.plan_template(Some(80)).is_err());
        }
    }

    /// `install_template_with` finds the offering an app came from by the id
    /// its plan carries, which is only sound while that id is the offering's
    /// own. Nothing in the type system says so, so it is asserted here.
    #[test]
    fn an_offering_maps_to_a_plan_that_carries_its_own_id() {
        for offering in offerings() {
            let template = offering.plan_template(None).expect("offering should map");
            assert_eq!(
                template.plan.id,
                offering.id(),
                "{} maps to a plan under a different id, so an install would look up the wrong app",
                offering.id()
            );
        }
    }

    /// An installed app takes its icon from the catalog entry its offering
    /// names. Imported apps were committed with no catalog id at all, so they
    /// installed without one; this fails if that comes back.
    #[test]
    fn every_offering_resolves_the_catalog_entry_its_icon_comes_from() {
        for offering in offerings() {
            assert!(
                crate::catalog::catalog_id(offering.catalog_name()).is_some(),
                "{} names catalog entry {:?}, which does not exist",
                offering.id(),
                offering.catalog_name()
            );
        }
    }

    /// Two sources feed one list, so an id colliding between them would make
    /// `offering` return whichever came first and silently shadow the other.
    #[test]
    fn no_two_offerings_share_an_id() {
        let mut ids: Vec<&str> = Vec::new();
        let offerings = offerings();
        for offering in &offerings {
            ids.push(offering.id());
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            ids.len(),
            "two offerings share an id: {ids:?}"
        );
    }
}
