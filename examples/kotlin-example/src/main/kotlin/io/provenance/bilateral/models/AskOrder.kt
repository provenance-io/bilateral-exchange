package io.provenance.bilateral.models

import com.fasterxml.jackson.annotation.JsonTypeInfo
import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import cosmos.base.v1beta1.CoinOuterClass.Coin

@JsonNaming(SnakeCaseStrategy::class)
data class AskOrder(
    val id: String,
    val askType: String,
    val owner: String,
    val collateral: AskCollateral,
    val descriptor: RequestDescriptor?
)

@JsonTypeInfo(use = JsonTypeInfo.Id.DEDUCTION)
sealed interface AskCollateral {
    /*
        {
          "id" : "fe3f6eaf-885f-4ea1-a2fe-a80e2fa745cd",
          "ask_type" : "coin_trade",
          "owner" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y",
          "collateral" : {
            "coin_trade" : {
              "base" : [ {
                "denom" : "nhash",
                "amount" : "50"
              } ],
              "quote" : [ {
                "denom" : "nhash",
                "amount" : "100"
              } ]
            }
          },
          "descriptor" : {
            "description" : "Example description",
            "effective_time" : "1655690320315894000",
            "attribute_requirement" : {
              "attributes" : [ "a.pb", "b.pio" ],
              "requirement_type" : "all"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class CoinTrade(val coinTrade: Body) : AskCollateral {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(
            val base: List<Coin>,
            val quote: List<Coin>,
        )
    }

    /*
        {
          "id" : "99ba4102-53b8-4d73-8096-e7194ac78604",
          "ask_type" : "marker_trade",
          "owner" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y",
          "collateral" : {
            "marker_trade" : {
              "address" : "tp1p3sl9tll0ygj3flwt5r2w0n6fx9p5ngqswjn5k",
              "denom" : "testcoin",
              "quote_per_share" : [ {
                "denom" : "nhash",
                "amount" : "50"
              } ],
              "removed_permissions" : [ {
                "address" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y",
                "permissions" : [ "admin", "deposit", "withdraw", "burn", "mint", "delete" ]
              } ]
            }
          },
          "descriptor" : {
            "description" : "Example description",
            "effective_time" : "1655690830145197000",
            "attribute_requirement" : {
              "attributes" : [ "attr.sc.pb", "other.pio" ],
              "requirement_type" : "any"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class MarkerTrade(val markerTrade: Body) : AskCollateral {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(
            val address: String,
            val denom: String,
            val quotePerShare: List<Coin>,
            val removedPermissions: List<MarkerAccessGrant>,
        )
    }

    /*
        SINGLE TRANSACTION TRADE:

        {
          "id" : "cbc40ee9-e79b-4763-be47-c2a442be8a3c",
          "ask_type" : "marker_share_sale",
          "owner" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y",
          "collateral" : {
            "marker_share_sale" : {
              "address" : "tp14hmzymp860j0ufhn37xltq2hwrnp440y9dmwyw",
              "denom" : "dankcoin",
              "remaining_shares" : "100",
              "quote_per_share" : [ {
                "denom" : "nhash",
                "amount" : "50"
              } ],
              "removed_permissions" : [ {
                "address" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y",
                "permissions" : [ "admin", "deposit", "withdraw", "burn", "mint", "delete" ]
              } ],
              "sale_type" : {
                "single_transaction" : {
                  "share_count" : "75"
                }
              }
            }
          },
          "descriptor" : {
            "description" : "Example description",
            "effective_time" : "1655691526779913000",
            "attribute_requirement" : {
              "attributes" : [ "attr.sc.pb", "other.pio" ],
              "requirement_type" : "none"
            }
          }
        }
     */

    /*
        MULTIPLE TRANSACTION TRADE:

        {
          "id" : "982457c3-18bd-4d8b-b5cf-1e09e9aa3bd8",
          "ask_type" : "marker_share_sale",
          "owner" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y",
          "collateral" : {
            "marker_share_sale" : {
              "address" : "tp1cvgk23rpp96300gmfxchfuq0y0arm6wtsh3v3a",
              "denom" : "noucoin",
              "remaining_shares" : "100",
              "quote_per_share" : [ {
                "denom" : "nhash",
                "amount" : "1000"
              } ],
              "removed_permissions" : [ {
                "address" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y",
                "permissions" : [ "admin", "deposit", "withdraw", "burn", "mint", "delete" ]
              } ],
              "sale_type" : {
                "multiple_transactions" : {
                  "remove_sale_share_threshold" : "75"
                }
              }
            }
          },
          "descriptor" : {
            "description" : "Example description",
            "effective_time" : "1655691957648311000",
            "attribute_requirement" : {
              "attributes" : [ "yolo.pb", "nou.pio" ],
              "requirement_type" : "all"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class MarkerShareSale(val markerShareSale: Body) : AskCollateral {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(
            val address: String,
            val denom: String,
            val remainingShares: String,
            val quotePerShare: List<Coin>,
            val removedPermissions: List<MarkerAccessGrant>,
            val saleType: ShareSaleType,
        )
    }

    /*
        {
          "id" : "ac0b4ec2-089d-45b0-b649-b2d43f1bcf5f",
          "ask_type" : "scope_trade",
          "owner" : "tp1yc4qpessmkxzc287te08kdku9gvta27fgr8m8y",
          "collateral" : {
            "scope_trade" : {
              "scope_address" : "scope1qz9puy0kqex5xfawzunfqrw25htquqr5ns",
              "quote" : [ {
                "denom" : "nhash",
                "amount" : "50000"
              } ]
            }
          },
          "descriptor" : {
            "description" : "Example description",
            "effective_time" : "1655692117958721000",
            "attribute_requirement" : {
              "attributes" : [ "abc.pb", "xyz.pio" ],
              "requirement_type" : "any"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class ScopeTrade(val scopeTrade: Body) : AskCollateral {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(
            val scopeAddress: String,
            val quote: List<Coin>,
        )
    }
}

data class MarkerAccessGrant(
    val address: String,
    val permissions: List<String>,
)
