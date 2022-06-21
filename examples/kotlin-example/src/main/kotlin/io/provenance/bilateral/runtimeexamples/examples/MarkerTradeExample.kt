package io.provenance.bilateral.runtimeexamples.examples

import io.provenance.bilateral.execute.CreateAsk
import io.provenance.bilateral.execute.CreateBid
import io.provenance.bilateral.execute.ExecuteMatch
import io.provenance.bilateral.models.RequestDescriptor
import io.provenance.bilateral.runtimeexamples.config.AppDependencies
import io.provenance.bilateral.runtimeexamples.examples.interfaces.Example
import io.provenance.bilateral.runtimeexamples.extensions.checkAsk
import io.provenance.bilateral.runtimeexamples.extensions.checkBid
import io.provenance.bilateral.runtimeexamples.extensions.verifyAskAndBidAreDeleted
import io.provenance.bilateral.runtimeexamples.extensions.wrapList
import io.provenance.bilateral.runtimeexamples.functions.coin
import io.provenance.bilateral.runtimeexamples.functions.createMarker
import io.provenance.bilateral.runtimeexamples.functions.grantMarkerAccess
import java.time.OffsetDateTime
import java.util.UUID

object MarkerTradeExample : Example {
    override fun start(deps: AppDependencies) {
        val markerDenom = deps.config.markerTradeConfig.markerDenom
        val client = deps.client
        createMarker(
            deps = deps,
            ownerAccount = deps.accounts.askerAccount,
            denomName = markerDenom,
            supply = 10,
        )
        grantMarkerAccess(
            deps = deps,
            markerAdminAccount = deps.accounts.askerAccount,
            markerDenom = markerDenom,
            grantAddress = deps.client.contractAddress,
        )
        val askUuid = UUID.randomUUID()
        println("Ask UUID: $askUuid")
        val createAsk = CreateAsk.newMarkerTrade(
            id = askUuid.toString(),
            denom = markerDenom,
            quotePerShare = coin(50, "nhash").wrapList(),
            descriptor = RequestDescriptor(description = "Example description", effectiveTime = OffsetDateTime.now()),
        )
        client.executeContract(signer = deps.accounts.askerAccount, executeMsg = createAsk)
        client.checkAsk(askUuid)
        val bidUuid = UUID.randomUUID()
        println("Bid uuid: $bidUuid")
        val createBid = CreateBid.newMarkerTrade(
            id = bidUuid.toString(),
            denom = markerDenom,
            descriptor = RequestDescriptor(description = "Example description", effectiveTime = OffsetDateTime.now()),
        )
        client.executeContract(
            signer = deps.accounts.bidderAccount,
            executeMsg = createBid,
            funds = coin(500, "nhash").wrapList(),
        )
        client.checkBid(bidUuid)
        client.executeContract(
            signer = deps.accounts.adminAccount,
            executeMsg = ExecuteMatch.new(askUuid.toString(), bidUuid.toString()),
        )
        client.verifyAskAndBidAreDeleted(askUuid, bidUuid)
        println("Successfully traded a marker")
    }
}

