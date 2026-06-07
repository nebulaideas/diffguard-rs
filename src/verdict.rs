use crate::error::DiffguardError;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewState {
    Approve,
    RequestChanges,
    Comment,
}

impl std::fmt::Display for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewState::Approve => write!(f, "APPROVE"),
            ReviewState::RequestChanges => write!(f, "REQUEST_CHANGES"),
            ReviewState::Comment => write!(f, "COMMENT"),
        }
    }
}

impl ReviewState {
    pub fn as_github_state(&self) -> &'static str {
        match self {
            ReviewState::Approve => "APPROVE",
            ReviewState::RequestChanges => "CHANGES_REQUESTED",
            ReviewState::Comment => "COMMENT",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub verdict: String,
    pub critical_bugs: u32,
    pub security_issues: u32,
}

pub fn parse_metadata_block(response: &str) -> Option<Verdict> {
    let re = Regex::new(
        r"\[DIFFGUARD_VERDICT_METADATA\][\s\S]*?Verdict:\s*(\w+)[\s\S]*?CriticalBugs:\s*(\d+)[\s\S]*?SecurityIssues:\s*(\d+)"
    )
    .ok()?;

    let caps = re.captures(response)?;
    Some(Verdict {
        verdict: caps.get(1)?.as_str().to_string(),
        critical_bugs: caps.get(2)?.as_str().parse().unwrap_or(0),
        security_issues: caps.get(3)?.as_str().parse().unwrap_or(0),
    })
}

pub fn evaluate_by_tags(response: &str) -> Verdict {
    let critical_re = Regex::new(r"\[Critical Bug\]|\[Critical\]").unwrap();
    let security_re = Regex::new(r"\[Security\]|\[Security Issue\]").unwrap();

    let critical_bugs = critical_re.find_iter(response).count() as u32;
    let security_issues = security_re.find_iter(response).count() as u32;

    Verdict {
        verdict: if critical_bugs > 0 || security_issues > 0 {
            "NEGATIVE".to_string()
        } else {
            "POSITIVE".to_string()
        },
        critical_bugs,
        security_issues,
    }
}

pub fn determine_review_state(verdict: &Verdict) -> ReviewState {
    // Asymmetric safety model: pessimistic signals are always trusted
    if verdict.verdict == "NEGATIVE"
        || verdict.security_issues > 0
        || verdict.critical_bugs > 2
    {
        ReviewState::RequestChanges
    } else if verdict.critical_bugs == 0 && verdict.security_issues == 0 {
        ReviewState::Approve
    } else {
        ReviewState::Comment
    }
}

pub fn parse_verdict(response: &str) -> Result<(Verdict, ReviewState), DiffguardError> {
    let verdict = parse_metadata_block(response).unwrap_or_else(|| evaluate_by_tags(response));

    if verdict.verdict != "POSITIVE" && verdict.verdict != "NEGATIVE" {
        return Err(DiffguardError::VerdictParse(format!(
            "Invalid verdict value: {}. Expected POSITIVE or NEGATIVE.",
            verdict.verdict
        )));
    }

    let state = determine_review_state(&verdict);
    Ok((verdict, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_positive() {
        let response = "Some review text\n\n[DIFFGUARD_VERDICT_METADATA]\nVerdict: POSITIVE\nCriticalBugs: 0\nSecurityIssues: 0";
        let verdict = parse_metadata_block(response).unwrap();
        assert_eq!(verdict.verdict, "POSITIVE");
        assert_eq!(verdict.critical_bugs, 0);
        assert_eq!(verdict.security_issues, 0);
        assert_eq!(determine_review_state(&verdict), ReviewState::Approve);
    }

    #[test]
    fn test_parse_negative() {
        let response = "Some review text\n\n[DIFFGUARD_VERDICT_METADATA]\nVerdict: NEGATIVE\nCriticalBugs: 0\nSecurityIssues: 0";
        let verdict = parse_metadata_block(response).unwrap();
        assert_eq!(determine_review_state(&verdict), ReviewState::RequestChanges);
    }

    #[test]
    fn test_parse_critical_gt_2() {
        let response = "[DIFFGUARD_VERDICT_METADATA]\nVerdict: POSITIVE\nCriticalBugs: 5\nSecurityIssues: 0";
        let verdict = parse_metadata_block(response).unwrap();
        assert_eq!(determine_review_state(&verdict), ReviewState::RequestChanges);
    }

    #[test]
    fn test_parse_security_gt_0() {
        let response = "[DIFFGUARD_VERDICT_METADATA]\nVerdict: POSITIVE\nCriticalBugs: 0\nSecurityIssues: 1";
        let verdict = parse_metadata_block(response).unwrap();
        assert_eq!(determine_review_state(&verdict), ReviewState::RequestChanges);
    }

    #[test]
    fn test_missing_metadata_fallback_to_tags() {
        let response = "Review found some issues.\n[Critical Bug] Race condition in handler\n[Security] SQL injection risk";
        let verdict = evaluate_by_tags(response);
        assert_eq!(verdict.critical_bugs, 1);
        assert_eq!(verdict.security_issues, 1);
        assert_eq!(determine_review_state(&verdict), ReviewState::RequestChanges);
    }

    #[test]
    fn test_clean_tag_fallback() {
        let response = "Everything looks good. No issues found.";
        let verdict = evaluate_by_tags(response);
        assert_eq!(verdict.critical_bugs, 0);
        assert_eq!(verdict.security_issues, 0);
        assert_eq!(determine_review_state(&verdict), ReviewState::Approve);
    }

    #[test]
    fn test_positive_with_minor_bugs() {
        let response = "[DIFFGUARD_VERDICT_METADATA]\nVerdict: POSITIVE\nCriticalBugs: 1\nSecurityIssues: 0";
        let verdict = parse_metadata_block(response).unwrap();
        assert_eq!(determine_review_state(&verdict), ReviewState::Comment);
    }
}
