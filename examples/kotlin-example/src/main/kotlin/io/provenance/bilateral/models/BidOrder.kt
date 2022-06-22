package io.provenance.bilateral.models

import com.fasterxml.jackson.annotation.JsonTypeInfo
import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import cosmos.base.v1beta1.CoinOuterClass.Coin

@JsonNaming(SnakeCaseStrategy::class)
data class BidOrder(
    val id: String,
    val bidType: String,
    val owner: String,
    val collateral: BidCollateral,
    val descriptor: RequestDescriptor?,
)

@JsonTypeInfo(use = JsonTypeInfo.Id.DEDUCTION)
sealed interface BidCollateral {
    /*
        {
          "id" : "c52eeda2-3224-4615-b5f9-e26a4a2f60a6",
          "bid_type" : "coin_trade",
          "owner" : "tp16v358yutrq9y24yny34j88yx7t48n6dn5c77v9",
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
            "effective_time" : "1655690324377129000",
            "attribute_requirement" : {
              "attributes" : [ "heyo.pb" ],
              "requirement_type" : "none"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class CoinTrade(val coinTrade: Body): BidCollateral {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(
            val base: List<Coin>,
            val quote: List<Coin>,
        )
    }

    /*
        {
          "id" : "d186dd8d-5068-4b62-a118-d33fcb2cd544",
          "bid_type" : "marker_trade",
          "owner" : "tp16v358yutrq9y24yny34j88yx7t48n6dn5c77v9",
          "collateral" : {
            "marker_trade" : {
              "address" : "tp1p3sl9tll0ygj3flwt5r2w0n6fx9p5ngqswjn5k",
              "denom" : "testcoin",
              "quote" : [ {
                "denom" : "nhash",
                "amount" : "500"
              } ]
            }
          },
          "descriptor" : {
            "description" : "Example description",
            "effective_time" : "1655690835272007000",
            "attribute_requirement" : {
              "attributes" : [ "whooaaaaahhhh.attr", "other.pio" ],
              "requirement_type" : "all"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class MarkerTrade(val markerTrade: Body): BidCollateral {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(
            val address: String,
            val denom: String,
            val quote: List<Coin>,
        )
    }

    /*
        SINGLE TRANSACTION TRADE:

        {
          "id" : "ee44d587-fd11-4803-b372-a820c41c4dfa",
          "bid_type" : "marker_share_sale",
          "owner" : "tp16v358yutrq9y24yny34j88yx7t48n6dn5c77v9",
          "collateral" : {
            "marker_share_sale" : {
              "address" : "tp14hmzymp860j0ufhn37xltq2hwrnp440y9dmwyw",
              "denom" : "dankcoin",
              "share_count" : "75",
              "quote" : [ {
                "denom" : "nhash",
                "amount" : "3750"
              } ]
            }
          },
          "descriptor" : {
            "description" : "Example description",
            "effective_time" : "1655691531898653000",
            "attribute_requirement" : {
              "attributes" : [ "hey.pb", "lol.pio" ],
              "requirement_type" : "any"
            }
          }
        }
     */

    /*
        MULTIPLE TRANSACTION TRADE:

        {
          "id" : "943b7f98-ffcd-4174-99a4-fda94f6a8f7c",
          "bid_type" : "marker_share_sale",
          "owner" : "tp16v358yutrq9y24yny34j88yx7t48n6dn5c77v9",
          "collateral" : {
            "marker_share_sale" : {
              "address" : "tp1cvgk23rpp96300gmfxchfuq0y0arm6wtsh3v3a",
              "denom" : "noucoin",
              "share_count" : "25",
              "quote" : [ {
                "denom" : "nhash",
                "amount" : "25000"
              } ]
            }
          },
          "descriptor" : {
            "description" : "Example description",
            "effective_time" : "1655691962780823000",
            "attribute_requirement" : {
              "attributes" : [ "aaaaaaahhhhh.pb", "helpme.pio" ],
              "requirement_type" : "none"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class MarkerShareSale(val markerShareSale: Body) : BidCollateral {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(
            val address: String,
            val denom: String,
            val shareCount: String,
            val quote: List<Coin>,
        )
    }

    /*
        {
          "id" : "721305c5-4a82-4174-81ed-225342f9e377",
          "bid_type" : "scope_trade",
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
            "effective_time" : "1655692123071177000",
            "attribute_requirement" : {
              "attributes" : [ "www.billywitchdoctor.com.pb", "jlksdfljksdfljk.pio" ],
              "requirement_type" : "all"
            }
          }
        }
     */
    @JsonNaming(SnakeCaseStrategy::class)
    data class ScopeTrade(val scopeTrade: Body) : BidCollateral {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(
            val scopeAddress: String,
            val quote: List<Coin>,
        )
    }
}
