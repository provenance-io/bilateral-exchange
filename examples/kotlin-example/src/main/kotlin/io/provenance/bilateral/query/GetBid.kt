package io.provenance.bilateral.query

import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import io.provenance.bilateral.interfaces.ContractQueryMsg

/*
    {
        "get_bid": {
            "id": "d3a7bcbe-f048-11ec-99b7-dbf1e63fa900"
        }
    }

    With Funds: [ ]
 */
@JsonNaming(SnakeCaseStrategy::class)
data class GetBid(val getBid: Body) : ContractQueryMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    data class Body(val id: String)

    companion object {
        fun new(id: String): GetBid = GetBid(getBid = Body(id = id))
    }
}

