package io.provenance.bilateral.interfaces

import com.fasterxml.jackson.databind.ObjectMapper
import cosmos.base.v1beta1.CoinOuterClass
import cosmwasm.wasm.v1beta1.Tx.MsgExecuteContract

interface ContractExecuteMsg : ContractMsg {
    fun toExecuteMsg(
        objectMapper: ObjectMapper,
        contractAddress: String,
        senderBech32Address: String,
        funds: List<CoinOuterClass.Coin>? = null,
    ): MsgExecuteContract = MsgExecuteContract.newBuilder().also { msg ->
        msg.msg = this.toJsonByteString(objectMapper)
        msg.contract = contractAddress
        msg.sender = senderBech32Address
        funds?.also { msg.addAllFunds(funds) }
    }.build()
}
