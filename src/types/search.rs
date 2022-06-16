use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Search {
    pub search_type: SearchType,
    pub page_size: Option<usize>,
    pub page_number: Option<usize>,
}
impl Search {
    pub fn all(page_size: Option<usize>, page_number: Option<usize>) -> Self {
        Self {
            search_type: SearchType::All,
            page_size,
            page_number,
        }
    }

    pub fn value_type<S: Into<String>>(
        value_type: S,
        page_size: Option<usize>,
        page_number: Option<usize>,
    ) -> Self {
        Self {
            search_type: SearchType::ValueType {
                value_type: value_type.into(),
            },
            page_size,
            page_number,
        }
    }

    pub fn id<S: Into<String>>(
        id: S,
        page_size: Option<usize>,
        page_number: Option<usize>,
    ) -> Self {
        Self {
            search_type: SearchType::Id { id: id.into() },
            page_size,
            page_number,
        }
    }

    pub fn owner<S: Into<String>>(
        owner: S,
        page_size: Option<usize>,
        page_number: Option<usize>,
    ) -> Self {
        Self {
            search_type: SearchType::Owner {
                owner: owner.into(),
            },
            page_size,
            page_number,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SearchResult<T> {
    pub results: Vec<T>,
    pub page_number: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    All,
    ValueType { value_type: String },
    Id { id: String },
    Owner { owner: String },
}
