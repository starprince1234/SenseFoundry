use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum CopyrightStatus {
    Verified,
    PartiallyVerified,
    Unverifiable,
    Rejected,
    LegalReviewRequired,
}

impl CopyrightStatus {
    /// Determines authorization status without treating URL access as a license grant.
    pub fn determine(
        license_declared: bool,
        has_isbn_or_doi: bool,
        _source_type: &str,
        url_accessible: bool,
    ) -> Self {
        if license_declared && has_isbn_or_doi {
            Self::Verified
        } else if license_declared && url_accessible {
            Self::PartiallyVerified
        } else if !license_declared && !has_isbn_or_doi {
            if url_accessible {
                Self::Rejected
            } else {
                Self::Unverifiable
            }
        } else {
            Self::LegalReviewRequired
        }
    }

    pub(crate) const fn authorization_flags(self) -> (bool, bool, bool) {
        match self {
            Self::Verified => (true, true, true),
            Self::PartiallyVerified => (true, false, false),
            Self::Unverifiable | Self::Rejected | Self::LegalReviewRequired => {
                (false, false, false)
            }
        }
    }
}
