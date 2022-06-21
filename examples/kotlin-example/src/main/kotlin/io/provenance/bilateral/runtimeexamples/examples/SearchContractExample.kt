package io.provenance.bilateral.runtimeexamples.examples

import cosmos.tx.v1beta1.ServiceOuterClass
import cosmwasm.wasm.v1.Tx
import io.provenance.bilateral.execute.CreateAsk
import io.provenance.bilateral.execute.CreateBid
import io.provenance.bilateral.query.BilateralRequestType
import io.provenance.bilateral.query.ContractSearchRequest
import io.provenance.bilateral.query.ContractSearchResult
import io.provenance.bilateral.query.ContractSearchType
import io.provenance.bilateral.runtimeexamples.config.AppDependencies
import io.provenance.bilateral.runtimeexamples.config.Singletons
import io.provenance.bilateral.runtimeexamples.examples.interfaces.Example
import io.provenance.bilateral.runtimeexamples.extensions.checkAsk
import io.provenance.bilateral.runtimeexamples.extensions.checkZeroResponseCode
import io.provenance.bilateral.runtimeexamples.extensions.wrapList
import io.provenance.bilateral.runtimeexamples.functions.coin
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.protobuf.extensions.toTxBody
import java.util.UUID

object SearchContractExample : Example {
    override fun start(deps: AppDependencies) {
        val askUuids = generateSequence { UUID.randomUUID() }.take(25).toSet()
        val bidUuids = generateSequence { UUID.randomUUID() }.take(25).toSet()
        check(askUuids.plus(bidUuids).size == 50) { "Both asks and bids should be unique" }
        val askMsgs = askUuids.map { askUuid ->
            val create = CreateAsk.newCoinTrade(id = askUuid.toString(), quote = coin(50, "nhash").wrapList()).toJsonByteString(
                Singletons.OBJECT_MAPPER
            )
            Tx.MsgExecuteContract.newBuilder().also { msg ->
                msg.msg = create
                msg.contract = deps.client.contractAddress
                msg.sender = deps.accounts.askerAccount.address()
                msg.addFunds(coin(50, "nhash"))
            }.build().toAny()
        }
        println("Broadcasting ALL the asks")
        deps.client.pbClient.estimateAndBroadcastTx(
            txBody = askMsgs.toTxBody(),
            signers = deps.accounts.askerAccount.let(::BaseReqSigner).wrapList(),
            mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK,
        ).checkZeroResponseCode()
        val bidMsgs = bidUuids.map { bidUuid ->
            val create = CreateBid.newCoinTrade(id = bidUuid.toString(), base = coin(50, "nhash").wrapList())
                .toJsonByteString(Singletons.OBJECT_MAPPER)
            Tx.MsgExecuteContract.newBuilder().also { msg ->
                msg.msg = create
                msg.contract = deps.client.contractAddress
                msg.sender = deps.accounts.bidderAccount.address()
                msg.addFunds(coin(50, "nhash"))
            }.build().toAny()
        }
        println("Broadcasting ALL the bids")
        deps.client.pbClient.estimateAndBroadcastTx(
            txBody = bidMsgs.toTxBody(),
            signers = deps.accounts.bidderAccount.let(::BaseReqSigner).wrapList(),
            mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK,
        ).checkZeroResponseCode()
        // Inline this function to keep it scoped here.  NEVER DO THIS IN PRODUCTION CODE. THIS CODE IS ONLY FOR TESTING LOL
        fun <T> ContractSearchResult<T>.checkSize(size: Int): ContractSearchResult<T> = this.also {
            check(this.results.size == size) { "Expected result list to be size of $size but it was ${this.results.size}" }
        }
        val allAsks = deps.client.searchAsks(
            ContractSearchRequest.newSearchAsks(
                searchType = ContractSearchType.all(),
                pageSize = 10,
                pageNumber = 2,
            )
        ).checkSize(10)
        val allBids = deps.client.searchBids(
            ContractSearchRequest.newSearchBids(
                searchType = ContractSearchType.all(),
                pageSize = 22,
                pageNumber = 1,
            )
        ).checkSize(22)
        val coinTradeAsks = deps.client.searchAsks(
            ContractSearchRequest.newSearchAsks(
                searchType = ContractSearchType.byType(BilateralRequestType.COIN_TRADE),
                pageSize = 2,
                pageNumber = 11,
            )
        ).checkSize(2)
        val coinTradeBids = deps.client.searchBids(
            ContractSearchRequest.newSearchBids(
                searchType = ContractSearchType.byType(BilateralRequestType.COIN_TRADE),
                pageSize = 17,
                pageNumber = 1,
            )
        ).checkSize(17)
        val idAsk = deps.client.searchAsks(
            ContractSearchRequest(
                searchType = ContractSearchType.byId(askUuids.first().toString()),
            ).searchAsks()
        ).checkSize(1)
        val idBid = deps.client.searchBids(
            ContractSearchRequest(
                searchType = ContractSearchType.byId(bidUuids.first().toString()),
            ).searchBids()
        ).checkSize(1)
        val ownerAsks = deps.client.searchAsks(
            ContractSearchRequest.newSearchAsks(
                searchType = ContractSearchType.byOwner(deps.accounts.askerAccount.address()),
                pageSize = 25,
                pageNumber = 1,
            )
        ).checkSize(25)
        val ownerBids = deps.client.searchBids(
            ContractSearchRequest.newSearchBids(
                searchType = ContractSearchType.byOwner(deps.accounts.bidderAccount.address()),
                pageSize = 3,
                pageNumber = 4,
            )
        ).checkSize(3)
        println("Successfully verified all searches")
    }
}

