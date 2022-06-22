package io.provenance.bilateral.execute

import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import cosmos.base.v1beta1.CoinOuterClass.Coin
import io.provenance.bilateral.interfaces.ContractExecuteMsg
import io.provenance.bilateral.models.RequestDescriptor
import io.provenance.bilateral.models.ShareSaleType

@JsonNaming(SnakeCaseStrategy::class)
data class CreateAsk(
    val createAsk: Body,
) : ContractExecuteMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    data class Body(
        val ask: Ask,
        val descriptor: RequestDescriptor?,
    )

    companion object {
        /*
            {
              "create_ask" : {
                "ask" : {
                  "coin_trade" : {
                    "id" : "fe3f6eaf-885f-4ea1-a2fe-a80e2fa745cd",
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
                    "attributes" : [ "something.pb" ],
                    "requirement_type": "any"
                  }
                }
              }
            }

            With Funds: [ {
              "denom" : "nhash",
              "amount" : "50"
            } ]
         */
        fun newCoinTrade(
            id: String,
            quote: List<Coin>,
            descriptor: RequestDescriptor? = null,
        ): CreateAsk = CreateAsk(
            createAsk = Body(
                ask = Ask.newCoinTrade(id, quote),
                descriptor = descriptor,
            )
        )

        /*
            {
              "create_ask" : {
                "ask" : {
                  "marker_trade" : {
                    "id" : "99ba4102-53b8-4d73-8096-e7194ac78604",
                    "denom" : "testcoin",
                    "quote_per_share" : [ {
                      "denom" : "nhash",
                      "amount" : "50"
                    } ]
                  }
                },
                "descriptor" : {
                  "description" : "Example description",
                  "effective_time" : "1655690830145197000",
                  "attribute_requirement" : {
                    "attributes" : [ "something.pb" ],
                    "requirement_type": "all"
                  }
                }
              }
            }

            With Funds: [ ]
         */
        /**
         * Note: A marker trade ask must be made AFTER the contract has been granted admin rights to the marker being
         * traded.
         */
        fun newMarkerTrade(
            id: String,
            denom: String,
            quotePerShare: List<Coin>,
            descriptor: RequestDescriptor? = null,
        ): CreateAsk = CreateAsk(
            createAsk = Body(
                ask = Ask.newMarkerTrade(id, denom, quotePerShare),
                descriptor = descriptor,
            )
        )

        /*
            SINGLE TRANSACTION TRADE:

            {
              "create_ask" : {
                "ask" : {
                  "marker_share_sale" : {
                    "id" : "cbc40ee9-e79b-4763-be47-c2a442be8a3c",
                    "denom" : "dankcoin",
                    "quote_per_share" : [ {
                      "denom" : "nhash",
                      "amount" : "50"
                    } ],
                    "share_sale_type" : {
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
                    "attributes" : [ "something.pb" ],
                    "requirement_type": "none"
                  }
                }
              }
            }

            With Funds: [ ]
         */
        /*
            MULTIPLE TRANSACTION TRADE:

            {
              "create_ask" : {
                "ask" : {
                  "marker_share_sale" : {
                    "id" : "982457c3-18bd-4d8b-b5cf-1e09e9aa3bd8",
                    "denom" : "noucoin",
                    "quote_per_share" : [ {
                      "denom" : "nhash",
                      "amount" : "1000"
                    } ],
                    "share_sale_type" : {
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
                    "attributes" : [ "a.pb", "b.pb", "c.pb" ],
                    "requirement_type": "all"
                  }
                }
              }
            }

            With Funds: [ ]
         */
        /**
         * Note: All marker share sales require that the contract be granted admin and withdraw rights on the marker
         * before the ask is created.  Recommended that this occurs in the same transaction.
         * Single share trades request that a specific number of shares be sold simultaneously in one bid match.
         * Multiple share trades allow any number of bids to be matched against the ask. The ask will only be deleted
         * in this circumstance once its shares have been depleted to zero (or if the share withdrawal limit has been
         * breached).
         */
        fun newMarkerShareSale(
            id: String,
            denom: String,
            quotePerShare: List<Coin>,
            shareSaleType: ShareSaleType,
            descriptor: RequestDescriptor? = null,
        ): CreateAsk = CreateAsk(
            createAsk = Body(
                ask = Ask.newMarkerShareSale(id, denom, quotePerShare, shareSaleType),
                descriptor = descriptor,
            )
        )

        /*
            {
              "create_ask" : {
                "ask" : {
                  "scope_trade" : {
                    "id" : "ac0b4ec2-089d-45b0-b649-b2d43f1bcf5f",
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
                    "attributes" : [ "a.pb", "b.pb", "c.pb" ],
                    "requirement_type": "none"
                  }
                }
              }
            }

            With Funds: [ ]
         */
        fun newScopeTrade(
            id: String,
            scopeAddress: String,
            quote: List<Coin>,
            descriptor: RequestDescriptor? = null,
        ): CreateAsk = CreateAsk(
            createAsk = Body(
                ask = Ask.newScopeTrade(id, scopeAddress, quote),
                descriptor = descriptor,
            )
        )
    }
}

sealed interface Ask {
    @JsonNaming(SnakeCaseStrategy::class)
    data class CoinTradeAsk(val coinTrade: Body) : Ask {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val id: String, val quote: List<Coin>)
    }

    @JsonNaming(SnakeCaseStrategy::class)
    data class MarkerTradeAsk(val markerTrade: Body) : Ask {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val id: String, val denom: String, val quotePerShare: List<Coin>)
    }

    @JsonNaming(SnakeCaseStrategy::class)
    data class MarkerShareSaleAsk(val markerShareSale: Body) : Ask {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val id: String, val denom: String, val quotePerShare: List<Coin>, val shareSaleType: ShareSaleType)
    }

    @JsonNaming(SnakeCaseStrategy::class)
    data class ScopeTradeAsk(val scopeTrade: Body) : Ask {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val id: String, val scopeAddress: String, val quote: List<Coin>)
    }

    companion object {
        fun newCoinTrade(id: String, quote: List<Coin>): Ask = CoinTradeAsk(
            coinTrade = CoinTradeAsk.Body(id, quote)
        )

        fun newMarkerTrade(id: String, denom: String, quotePerShare: List<Coin>): Ask = MarkerTradeAsk(
            markerTrade = MarkerTradeAsk.Body(id, denom, quotePerShare)
        )

        fun newMarkerShareSale(id: String, denom: String, quotePerShare: List<Coin>, shareSaleType: ShareSaleType): Ask = MarkerShareSaleAsk(
            markerShareSale = MarkerShareSaleAsk.Body(id, denom, quotePerShare, shareSaleType)
        )

        fun newScopeTrade(id: String, scopeAddress: String, quote: List<Coin>): Ask = ScopeTradeAsk(
            scopeTrade = ScopeTradeAsk.Body(id, scopeAddress, quote)
        )
    }
}
