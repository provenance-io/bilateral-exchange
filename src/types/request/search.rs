use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Search {
    pub search_type: SearchType,
    pub page_size: Option<Uint128>,
    pub page_number: Option<Uint128>,
}
impl Search {
    pub fn all(page_size: Option<u128>, page_number: Option<u128>) -> Self {
        Self {
            search_type: SearchType::All,
            page_size: page_size.map(Uint128::new),
            page_number: page_number.map(Uint128::new),
        }
    }

    pub fn value_type<S: Into<String>>(
        value_type: S,
        page_size: Option<u128>,
        page_number: Option<u128>,
    ) -> Self {
        Self {
            search_type: SearchType::ValueType {
                value_type: value_type.into(),
            },
            page_size: page_size.map(Uint128::new),
            page_number: page_number.map(Uint128::new),
        }
    }

    pub fn id<S: Into<String>>(id: S, page_size: Option<u128>, page_number: Option<u128>) -> Self {
        Self {
            search_type: SearchType::Id { id: id.into() },
            page_size: page_size.map(Uint128::new),
            page_number: page_number.map(Uint128::new),
        }
    }

    pub fn owner<S: Into<String>>(
        owner: S,
        page_size: Option<u128>,
        page_number: Option<u128>,
    ) -> Self {
        Self {
            search_type: SearchType::Owner {
                owner: owner.into(),
            },
            page_size: page_size.map(Uint128::new),
            page_number: page_number.map(Uint128::new),
        }
    }
}

// A helper trait to constrain SearchResult's T to a specific type.  Otherwise, the compiler goes
// absolutely bananas with serde and adds floating point code to the output wasm, causing it to be
// rejected by the Provenance Blockchain!!
pub trait Searchable {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SearchResult<T>
where
    T: Searchable,
{
    pub results: Vec<T>,
    pub page_number: Uint128,
    pub page_size: Uint128,
    pub total_pages: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    All,
    ValueType { value_type: String },
    Id { id: String },
    Owner { owner: String },
}
