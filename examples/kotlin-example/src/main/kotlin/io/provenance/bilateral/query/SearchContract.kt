package io.provenance.bilateral.query

import com.fasterxml.jackson.annotation.JsonValue
import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import io.provenance.bilateral.interfaces.ContractQueryMsg

/**
 * See ContractSearchType for JSON payloads for each different type of ask search.
 */
@JsonNaming(SnakeCaseStrategy::class)
data class SearchAsks(val searchAsks: Body) : ContractQueryMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    data class Body(val search: ContractSearchRequest)
}

/**
 * See ContractSearchType for JSON payloads for each different type of bid search.
 */
@JsonNaming(SnakeCaseStrategy::class)
data class SearchBids(val searchBids: Body) : ContractQueryMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    data class Body(val search: ContractSearchRequest)
}

@JsonNaming(SnakeCaseStrategy::class)
data class ContractSearchRequest(
    val searchType: ContractSearchType,
    val pageSize: String? = null,
    val pageNumber: String? = null,
) {
    companion object {
        fun newSearchAsks(
            searchType: ContractSearchType,
            pageSize: Int? = null,
            pageNumber: Int? = null,
        ): SearchAsks = ContractSearchRequest(searchType, pageSize?.toString(), pageNumber?.toString()).searchAsks()

        fun newSearchBids(
            searchType: ContractSearchType,
            pageSize: Int? = null,
            pageNumber: Int? = null,
        ): SearchBids = ContractSearchRequest(searchType, pageSize?.toString(), pageNumber?.toString()).searchBids()
    }

    fun searchAsks(): SearchAsks = SearchAsks(SearchAsks.Body(this))
    fun searchBids(): SearchBids = SearchBids(SearchBids.Body(this))
}

sealed interface ContractSearchType {
    /*
        {
          "search_asks" : {
            "search" : {
              "search_type" : "all",
              "page_size" : "10",
              "page_number" : "2"
            }
          }
        }
     */

    /*
        {
          "search_bids" : {
            "search" : {
              "search_type" : "all",
              "page_size" : "22",
              "page_number" : "3"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    object All : ContractSearchType {
        @JsonValue
        fun serializeAs(): String = "all"
    }

    /*
        {
          "search_asks" : {
            "search" : {
              "search_type" : {
                "value_type" : {
                  "value_type" : "coin_trade"
                }
              },
              "page_size" : "2",
              "page_number" : "11"
            }
          }
        }
     */

    /*
        {
          "search_bids" : {
            "search" : {
              "search_type" : {
                "value_type" : {
                  "value_type" : "coin_trade"
                }
              },
              "page_size" : "17",
              "page_number" : "1"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class Type(val valueType: Body) : ContractSearchType {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val valueType: String)
    }

    /*
        {
          "search_asks" : {
            "search" : {
              "search_type" : {
                "id" : {
                  "id" : "fb40d8ea-943d-41e0-8475-c149c17f2e42"
                }
              }
            }
          }
        }
     */

    /*
        {
          "search_bids" : {
            "search" : {
              "search_type" : {
                "id" : {
                  "id" : "038c3f95-981f-4bd4-a594-9b978c13cbe1"
                }
              }
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class Id(val id: Body) : ContractSearchType {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val id: String)
    }

    /*
        {
          "search_asks" : {
            "search" : {
              "search_type" : {
                "owner" : {
                  "owner" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y"
                }
              },
              "page_size" : "25",
              "page_number" : "1"
            }
          }
        }
     */

    /*
        {
          "search_bids" : {
            "search" : {
              "search_type" : {
                "owner" : {
                  "owner" : "tp16v358yutrq9y24yny34j88yx7t48n6dn5c77v9"
                }
              },
              "page_size" : "3",
              "page_number" : "4"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class Owner(val owner: Body) : ContractSearchType {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val owner: String)
    }

    companion object {
        fun all(): ContractSearchType = All
        fun byType(type: BilateralRequestType): ContractSearchType = Type(Type.Body(type.contractName))
        fun byId(id: String): ContractSearchType = Id(Id.Body(id))
        fun byOwner(owner: String): ContractSearchType = Owner(Owner.Body(owner))
    }
}

@JsonNaming(SnakeCaseStrategy::class)
data class ContractSearchResult<T>(
    val results: List<T>,
    val pageNumber: Int,
    val pageSize: Int,
    val totalPages: Int,
)

enum class BilateralRequestType(val contractName: String) {
    COIN_TRADE("coin_trade"),
    MARKER_TRADE("marker_trade"),
    MARKER_SHARE_SALE("marker_share_sale"),
    SCOPE_TRADE("scope_trade"),
}
