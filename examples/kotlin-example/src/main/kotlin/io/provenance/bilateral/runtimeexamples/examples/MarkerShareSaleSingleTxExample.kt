package io.provenance.bilateral.runtimeexamples.examples

import io.provenance.bilateral.execute.CreateAsk
import io.provenance.bilateral.execute.CreateBid
import io.provenance.bilateral.execute.ExecuteMatch
import io.provenance.bilateral.models.RequestDescriptor
import io.provenance.bilateral.models.ShareSaleType
import io.provenance.bilateral.runtimeexamples.config.AppDependencies
import io.provenance.bilateral.runtimeexamples.examples.interfaces.Example
import io.provenance.bilateral.runtimeexamples.extensions.checkAsk
import io.provenance.bilateral.runtimeexamples.extensions.checkBid
import io.provenance.bilateral.runtimeexamples.extensions.verifyAskAndBidAreDeleted
import io.provenance.bilateral.runtimeexamples.extensions.wrapList
import io.provenance.bilateral.runtimeexamples.functions.coin
import io.provenance.bilateral.runtimeexamples.functions.createMarker
import io.provenance.bilateral.runtimeexamples.functions.grantMarkerAccess
import io.provenance.marker.v1.Access
import java.time.OffsetDateTime
import java.util.UUID

object MarkerShareSaleSingleTxExample : Example {
    override fun start(deps: AppDependencies) {
        val markerDenom = deps.config.markerShareSaleSingleTxConfig.markerDenom
        val shareCount = deps.config.markerShareSaleSingleTxConfig.shareCount
        val shareSaleAmount = deps.config.markerShareSaleSingleTxConfig.shareSaleAmount
        val client = deps.client
        check(shareCount >= shareSaleAmount) { "Cannot sell more shares than exist in the marker!" }
        createMarker(
            deps = deps,
            ownerAccount = deps.accounts.askerAccount,
            denomName = markerDenom,
            supply = shareCount
        )
        grantMarkerAccess(
            deps = deps,
            markerAdminAccount = deps.accounts.askerAccount,
            markerDenom =  markerDenom,
            grantAddress = client.contractAddress,
            permissions = listOf(Access.ACCESS_ADMIN, Access.ACCESS_WITHDRAW),
        )
        val askUuid = UUID.randomUUID()
        val createAsk = CreateAsk.newMarkerShareSale(
            id = askUuid.toString(),
            denom = markerDenom,
            quotePerShare = coin(50, "nhash").wrapList(),
            shareSaleType = ShareSaleType.single(shareSaleAmount.toString()),
            descriptor = RequestDescriptor("Example description", OffsetDateTime.now())
        )
        client.executeContract(deps.accounts.askerAccount, createAsk)
        client.checkAsk(askUuid)
        val bidUuid = UUID.randomUUID()
        val createBid = CreateBid.newMarkerShareSale(
            id = bidUuid.toString(),
            denom = markerDenom,
            shareCount = shareSaleAmount.toString(),
            descriptor = RequestDescriptor("Example description", OffsetDateTime.now())
        )
        client.executeContract(
            signer = deps.accounts.bidderAccount,
            executeMsg = createBid,
            funds = coin(50 * shareSaleAmount, "nhash").wrapList(),
        )
        client.checkBid(bidUuid)
        val executeMatch = ExecuteMatch.new(askUuid.toString(), bidUuid.toString())
        client.executeContract(deps.accounts.adminAccount, executeMatch)
        client.verifyAskAndBidAreDeleted(askUuid = askUuid, bidUuid = bidUuid)
        println("Successfully executed a single marker share sale")
    }
}
