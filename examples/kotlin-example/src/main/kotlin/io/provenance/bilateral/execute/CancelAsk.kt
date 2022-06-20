package io.provenance.bilateral.execute

import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import io.provenance.bilateral.interfaces.ContractExecuteMsg

/*
    {
        "cancel_ask": {
            "id": "93140e28-f048-11ec-9785-cf8be8abc059"
        }
    }

    With Funds: [ ]
 */
@JsonNaming(SnakeCaseStrategy::class)
data class CancelAsk(val cancelAsk: Body) : ContractExecuteMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    data class Body(val id: String)

    companion object {
        fun new(id: String): CancelAsk = CancelAsk(cancelAsk = Body(id = id))
    }
}
