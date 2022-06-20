package io.provenance.bilateral.execute

import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import io.provenance.bilateral.interfaces.ContractExecuteMsg

/*
    {
      "execute_match" : {
        "ask_id" : "fe3f6eaf-885f-4ea1-a2fe-a80e2fa745cd",
        "bid_id" : "c52eeda2-3224-4615-b5f9-e26a4a2f60a6"
      }
    }

    With Funds: [ ]
 */
/**
 * An execute match contract route call must be made by the contract admin address.
 */
@JsonNaming(SnakeCaseStrategy::class)
data class ExecuteMatch(val executeMatch: Body) : ContractExecuteMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    data class Body(val askId: String, val bidId: String)

    companion object {
        fun new(askId: String, bidId: String): ExecuteMatch = ExecuteMatch(executeMatch = Body(askId, bidId))
    }
}
