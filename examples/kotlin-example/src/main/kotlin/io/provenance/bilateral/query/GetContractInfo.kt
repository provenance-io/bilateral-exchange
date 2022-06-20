package io.provenance.bilateral.query

import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import io.provenance.bilateral.interfaces.ContractQueryMsg

/*
    {
        "get_contract_info": { }
    }

    With Funds: [ ]
 */
@JsonNaming(SnakeCaseStrategy::class)
data class GetContractInfo(val getContractInfo: Body) : ContractQueryMsg {
    @JsonNaming(SnakeCaseStrategy::class)
    class Body

    companion object {
        fun new(): GetContractInfo = GetContractInfo(getContractInfo = Body())
    }
}
