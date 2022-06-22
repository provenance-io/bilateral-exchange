use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RequestDescriptor {
    pub description: Option<String>,
    pub effective_time: Option<Timestamp>,
    pub attribute_requirement: Option<AttributeRequirement>,
}
impl RequestDescriptor {
    pub fn new_none() -> Self {
        Self {
            description: None,
            effective_time: None,
            attribute_requirement: None,
        }
    }

    pub fn basic<S: Into<String>>(description: S) -> Self {
        Self {
            description: Some(description.into()),
            ..Self::new_none()
        }
    }

    pub fn new_populated_attributes<S: Into<String>>(
        description: S,
        attribute_requirement: AttributeRequirement,
    ) -> Self {
        Self {
            description: Some(description.into()),
            effective_time: Some(Timestamp::default()),
            attribute_requirement: Some(attribute_requirement),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AttributeRequirement {
    pub attributes: Vec<String>,
    pub requirement_type: AttributeRequirementType,
}
impl AttributeRequirement {
    pub fn all<S: Into<String>>(attributes: &[S]) -> Self
    where
        S: Clone,
    {
        Self::new(attributes, AttributeRequirementType::All)
    }

    pub fn any<S: Into<String>>(attributes: &[S]) -> Self
    where
        S: Clone,
    {
        Self::new(attributes, AttributeRequirementType::Any)
    }

    pub fn none<S: Into<String>>(attributes: &[S]) -> Self
    where
        S: Clone,
    {
        Self::new(attributes, AttributeRequirementType::None)
    }

    fn new<S: Into<String>>(attributes: &[S], requirement_type: AttributeRequirementType) -> Self
    where
        S: Clone,
    {
        Self {
            attributes: attributes.iter().cloned().map(|s| s.into()).collect(),
            requirement_type,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttributeRequirementType {
    // Requires that all specified AttributeRequirement.attributes values are present in an account
    All,
    // Requires that at least one of the specified AttributeRequirement.attributes values are
    // present in an account
    Any,
    // Requires that none of the specified AttributeRequirement.attributes values are present in an
    // account
    None,
}
