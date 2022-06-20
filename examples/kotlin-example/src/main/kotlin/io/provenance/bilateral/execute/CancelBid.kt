package io.provenance.bilateral.execute

import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import io.provenance.bilateral.interfaces.ContractExecuteMsg

/*
    {
        "cancel_bid": {
            "id": "af1e05f6-f048-11ec-9370-77e7f4c8d8ec"
        }
    }

    With Funds: [ ]
 */
@JsonNaming(SnakeCaseStrategy::class)
data class CancelBid(val cancelBid: Body) : ContractExecuteMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    data class Body(val id: String)

    companion object {
        fun new(id: String): CancelBid = CancelBid(cancelBid = Body(id = id))
    }
}
