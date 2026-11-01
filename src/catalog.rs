//! Reviewed, compile-time artifact pins. No participant-supplied catalog or update.
use crate::model::*;
use serde::Deserialize;
use std::sync::OnceLock;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Catalog {
    schema_version: u32,
    pub catalog_version: String,
    pub artifacts: Vec<Pin>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pin {
    pub tool_id: String,
    pub tool_version: String,
    pub rule_id: String,
    pub rule_version: String,
    pub architecture: String,
    pub bytes: u64,
    pub sha256: String,
    acquisition_url: String,
    release_url: String,
    approval: String,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn parse(input: &str) -> Result<Catalog, GapReason> {
    if input.len() > 65536 {
        return Err(GapReason::InvalidMetadata);
    }
    let c: Catalog = serde_json::from_str(input).map_err(|_| GapReason::InvalidMetadata)?;
    if c.schema_version != 1
        || !identifier(&c.catalog_version)
        || c.artifacts.is_empty()
        || c.artifacts.len() > 32
    {
        return Err(GapReason::InvalidMetadata);
    }
    for (index, p) in c.artifacts.iter().enumerate() {
        if p.tool_id != "codex-cli" || p.architecture != "x64" || p.approval != "diagnostic-closure"
            || ![&p.tool_version,&p.rule_id,&p.rule_version].iter().all(|v| identifier(v))
            || p.bytes == 0 || p.bytes > 512*1024*1024 || p.sha256.len()!=64
            || !p.sha256.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || p.acquisition_url != format!("https://github.com/openai/codex/releases/download/rust-v{}/codex-x86_64-pc-windows-msvc.exe",p.tool_version)
            || p.release_url != format!("https://github.com/openai/codex/releases/tag/rust-v{}",p.tool_version)
            || c.artifacts[..index].iter().any(|other| other.rule_id==p.rule_id || other.sha256==p.sha256)
        {return Err(GapReason::InvalidMetadata);}
    }
    Ok(c)
}
pub(crate) fn get() -> Result<&'static Catalog, GapReason> {
    static VALUE: OnceLock<Result<Catalog, GapReason>> = OnceLock::new();
    VALUE
        .get_or_init(|| parse(include_str!("../catalog/windows-artifacts.json")))
        .as_ref()
        .map_err(|e| *e)
}
pub(crate) fn public_match(identity: &IdentityAssessment) -> Option<(&'static str, &'static str)> {
    if !matches!(
        identity.class,
        EvidenceClass::Corroborated | EvidenceClass::ConfirmedSupported
    ) {
        return None;
    }
    let tool = identity.tool.as_ref()?;
    let rule = identity.rule_match.as_ref()?;
    let catalog = get().ok()?;
    if tool.category != ToolCategory::CodingAssistant
        || rule.approval != RuleApproval::DiagnosticClosure
        || rule.catalog_version != catalog.catalog_version
    {
        return None;
    }
    catalog
        .artifacts
        .iter()
        .find(|p| {
            p.tool_id == tool.id && p.rule_id == rule.rule_id && p.rule_version == rule.rule_version
        })
        .map(|p| (p.rule_id.as_str(), catalog.catalog_version.as_str()))
}
#[cfg(windows)]
pub(crate) fn assessment(pin: &Pin, catalog: &Catalog) -> IdentityAssessment {
    IdentityAssessment {
        tool: Some(ToolIdentity {
            id: pin.tool_id.clone(),
            category: ToolCategory::CodingAssistant,
        }),
        class: EvidenceClass::Corroborated,
        rule_match: Some(RuleMatch {
            catalog_version: catalog.catalog_version.clone(),
            rule_id: pin.rule_id.clone(),
            rule_version: pin.rule_version.clone(),
            approval: RuleApproval::DiagnosticClosure,
        }),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_pins_are_bounded_distinct_and_from_the_named_upstream() {
        assert!(get().is_ok());
        let original = include_str!("../catalog/windows-artifacts.json");
        for invalid in [
            original.replace("https://github.com/openai/", "https://untrusted.invalid/"),
            original.replace("diagnostic-closure", "unreviewed"),
            original.replace("\"schema_version\": 1", "\"schema_version\": 2"),
        ] {
            assert!(parse(&invalid).is_err());
        }
    }
}
