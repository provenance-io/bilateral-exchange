package io.provenance.bilateral.query

import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import io.provenance.bilateral.interfaces.ContractQueryMsg

/*
    {
        "get_ask": {
            "id": "c3fd577e-f048-11ec-8b83-530492d446a5"
        }
    }

    With Funds: [ ]
 */
@JsonNaming(SnakeCaseStrategy::class)
data class GetAsk(val getAsk: Body) : ContractQueryMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    data class Body(val id: String)

    companion object {
        fun new(id: String): GetAsk = GetAsk(getAsk = Body(id = id))
    }
}
