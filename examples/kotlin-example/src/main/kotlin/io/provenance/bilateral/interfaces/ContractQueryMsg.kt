package io.provenance.bilateral.interfaces

import com.fasterxml.jackson.databind.ObjectMapper
import cosmwasm.wasm.v1beta1.QueryOuterClass.QuerySmartContractStateRequest

interface ContractQueryMsg : ContractMsg {
    fun toQueryMsg(
        objectMapper: ObjectMapper,
        contractAddress: String,
    ): QuerySmartContractStateRequest = QuerySmartContractStateRequest.newBuilder().also { msg ->
        msg.queryData = this.toJsonByteString(objectMapper)
        msg.address = contractAddress
    }.build()
}
