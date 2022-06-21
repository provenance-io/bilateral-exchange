package io.provenance.bilateral.client

import com.fasterxml.jackson.databind.ObjectMapper
import cosmos.base.v1beta1.CoinOuterClass.Coin
import cosmos.tx.v1beta1.ServiceOuterClass.BroadcastMode
import cosmwasm.wasm.v1.QueryOuterClass
import cosmwasm.wasm.v1.Tx
import io.provenance.bilateral.interfaces.ContractExecuteMsg
import io.provenance.bilateral.interfaces.ContractQueryMsg
import io.provenance.bilateral.models.AskOrder
import io.provenance.bilateral.models.BidOrder
import io.provenance.bilateral.models.ContractInfo
import io.provenance.bilateral.query.ContractSearchResult
import io.provenance.bilateral.query.GetAsk
import io.provenance.bilateral.query.GetBid
import io.provenance.bilateral.query.GetContractInfo
import io.provenance.bilateral.query.SearchAsks
import io.provenance.bilateral.query.SearchBids
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.grpc.PbClient
import io.provenance.client.grpc.Signer
import io.provenance.client.protobuf.extensions.queryWasm
import io.provenance.client.protobuf.extensions.resolveAddressForName
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.protobuf.extensions.toTxBody

class BilateralContractClient(
    val pbClient: PbClient,
    val objectMapper: ObjectMapper,
    val contractName: String,
) {
    val contractAddress by lazy { pbClient.nameClient.resolveAddressForName(contractName) }

    fun getAsk(id: String): AskOrder = queryContract(GetAsk.new(id))

    fun getBid(id: String): BidOrder = queryContract(GetBid.new(id))

    fun searchAsks(searchAsks: SearchAsks): ContractSearchResult<AskOrder> = queryContract(searchAsks)

    fun searchBids(searchBids: SearchBids): ContractSearchResult<BidOrder> = queryContract(searchBids)

    fun getContractInfo(): ContractInfo = queryContract(GetContractInfo.new())

    fun executeContract(
        signer: Signer,
        executeMsg: ContractExecuteMsg,
        funds: List<Coin> = emptyList(),
        broadcastMode: BroadcastMode = BroadcastMode.BROADCAST_MODE_BLOCK,
    ) {
        val msg = Tx.MsgExecuteContract.newBuilder().also { msg ->
            msg.msg = executeMsg.toJsonByteString(objectMapper)
            msg.contract = contractAddress
            msg.sender = signer.address()
            msg.addAllFunds(funds)
        }.build()
        pbClient.estimateAndBroadcastTx(
            txBody = msg.toAny().toTxBody(),
            signers = listOf(BaseReqSigner(signer = signer)),
            mode = broadcastMode,
        ).also { response ->
            if (response.txResponse.code != 0) {
                throw IllegalStateException("FAILED: ${response.txResponse.rawLog}")
            } else {
                println("Response log: ${response.txResponse.rawLog}\n\n")
            }
        }
    }

    private inline fun <T : ContractQueryMsg, reified U : Any> queryContract(query: T): U {
        println("Querying contract [$contractName] at address [$contractAddress] for query type ${query::class.simpleName}")
        return pbClient.wasmClient.queryWasm(
            QueryOuterClass.QuerySmartContractStateRequest.newBuilder().also { req ->
                req.address = contractAddress
                req.queryData = query.toJsonByteString(objectMapper)
            }.build()
        ).data.toByteArray().let { bytes -> objectMapper.readValue(bytes, U::class.java) }
    }
}
