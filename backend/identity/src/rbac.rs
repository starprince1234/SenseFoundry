use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Contributor,
    CorpusAdmin,
    Editor,
    Linguist,
    Arbiter,
    ModelEngineer,
    DataGovernance,
    Legal,
    SecurityAdmin,
    PublicationManager,
    Ops,
    ProjectLead,
}

impl Role {
    pub const ALL: [Self; 13] = [
        Self::User,
        Self::Contributor,
        Self::CorpusAdmin,
        Self::Editor,
        Self::Linguist,
        Self::Arbiter,
        Self::ModelEngineer,
        Self::DataGovernance,
        Self::Legal,
        Self::SecurityAdmin,
        Self::PublicationManager,
        Self::Ops,
        Self::ProjectLead,
    ];

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Self::User),
            "contributor" => Some(Self::Contributor),
            "corpus_admin" => Some(Self::CorpusAdmin),
            "editor" => Some(Self::Editor),
            "linguist" => Some(Self::Linguist),
            "arbiter" => Some(Self::Arbiter),
            "model_engineer" => Some(Self::ModelEngineer),
            "data_governance" => Some(Self::DataGovernance),
            "legal" => Some(Self::Legal),
            "security_admin" => Some(Self::SecurityAdmin),
            "publication_manager" => Some(Self::PublicationManager),
            "ops" => Some(Self::Ops),
            "project_lead" => Some(Self::ProjectLead),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Contributor => "contributor",
            Self::CorpusAdmin => "corpus_admin",
            Self::Editor => "editor",
            Self::Linguist => "linguist",
            Self::Arbiter => "arbiter",
            Self::ModelEngineer => "model_engineer",
            Self::DataGovernance => "data_governance",
            Self::Legal => "legal",
            Self::SecurityAdmin => "security_admin",
            Self::PublicationManager => "publication_manager",
            Self::Ops => "ops",
            Self::ProjectLead => "project_lead",
        }
    }

    pub fn satisfies(self, required: Self) -> bool {
        self == required || self == Self::ProjectLead
    }
}

impl fmt::Display for Role {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Role {
    type Err = ParseRoleError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Role::from_str(value).ok_or_else(|| ParseRoleError(value.to_owned()))
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown role: {0}")]
pub struct ParseRoleError(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    ReadPublishedDictionary,
    SubmitCorpus,
    VerifyCorpus,
    EditDictionary,
    ReviewLinguistics,
    ArbitrateReviews,
    ManageModels,
    GovernData,
    ReviewLegal,
    ManageIdentity,
    PublishEditions,
    OperatePlatform,
    LeadProject,
    ListUsers,
    ManageUserRoles,
}

pub const fn role_has_permission(role: Role, permission: Permission) -> bool {
    if matches!(role, Role::ProjectLead) {
        return true;
    }

    match permission {
        Permission::ReadPublishedDictionary => true,
        Permission::SubmitCorpus => matches!(role, Role::Contributor),
        Permission::VerifyCorpus | Permission::ListUsers => {
            matches!(role, Role::CorpusAdmin | Role::SecurityAdmin)
        }
        Permission::EditDictionary => matches!(role, Role::Editor),
        Permission::ReviewLinguistics => matches!(role, Role::Linguist),
        Permission::ArbitrateReviews => matches!(role, Role::Arbiter),
        Permission::ManageModels => matches!(role, Role::ModelEngineer),
        Permission::GovernData => matches!(role, Role::DataGovernance),
        Permission::ReviewLegal => matches!(role, Role::Legal),
        Permission::ManageIdentity | Permission::ManageUserRoles => {
            matches!(role, Role::SecurityAdmin)
        }
        Permission::PublishEditions => matches!(role, Role::PublicationManager),
        Permission::OperatePlatform => matches!(role, Role::Ops),
        Permission::LeadProject => false,
    }
}

pub fn has_permission(roles: &[Role], permission: Permission) -> bool {
    roles
        .iter()
        .copied()
        .any(|role| role_has_permission(role, permission))
}
