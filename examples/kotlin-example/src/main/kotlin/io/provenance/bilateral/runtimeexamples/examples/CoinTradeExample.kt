package io.provenance.bilateral.runtimeexamples.examples

import io.provenance.bilateral.execute.CreateAsk
import io.provenance.bilateral.execute.CreateBid
import io.provenance.bilateral.execute.ExecuteMatch
import io.provenance.bilateral.models.AttributeRequirement
import io.provenance.bilateral.models.AttributeRequirementType
import io.provenance.bilateral.models.RequestDescriptor
import io.provenance.bilateral.runtimeexamples.config.AppDependencies
import io.provenance.bilateral.runtimeexamples.examples.interfaces.Example
import io.provenance.bilateral.runtimeexamples.extensions.checkAsk
import io.provenance.bilateral.runtimeexamples.extensions.checkBid
import io.provenance.bilateral.runtimeexamples.extensions.verifyAskAndBidAreDeleted
import io.provenance.bilateral.runtimeexamples.functions.coins
import java.time.OffsetDateTime
import java.util.UUID

object CoinTradeExample : Example {
    override fun start(deps: AppDependencies) {
        val quote = coins(denom = deps.config.coinTradeConfig.quoteDenom, amount = deps.config.coinTradeConfig.quoteAmount)
        val base = coins(denom = deps.config.coinTradeConfig.baseDenom, amount = deps.config.coinTradeConfig.baseAmount)
        val client = deps.client
        val askUuid = UUID.randomUUID()
        println("Creating ask with UUID: $askUuid")
        val createAsk = CreateAsk.newCoinTrade(
            id = askUuid.toString(),
            quote = quote,
            descriptor = RequestDescriptor(
                description = "Example description",
                effectiveTime = OffsetDateTime.now(),
                attributeRequirement = AttributeRequirement.new(listOf("a.pb", "b.pb"), AttributeRequirementType.NONE),
            )
        )
        client.executeContract(
            signer = deps.accounts.askerAccount,
            executeMsg = createAsk,
            funds = base,
        )
        // Verify that the ask order is available
        client.checkAsk(askUuid, createAsk.createAsk.descriptor?.effectiveTime)
        val bidUuid = UUID.randomUUID()
        println("Creating bid with UUID: $bidUuid")
        val createBid = CreateBid.newCoinTrade(
            id = bidUuid.toString(),
            base = base,
            descriptor = RequestDescriptor(
                description = "Example description",
                effectiveTime = OffsetDateTime.now(),
                attributeRequirement = AttributeRequirement.new(listOf("c.pb"), AttributeRequirementType.NONE),
            ),
        )
        client.executeContract(
            signer = deps.accounts.bidderAccount,
            executeMsg = createBid,
            funds = quote,
        )
        // Verify that the bid order is available
        client.checkBid(bidUuid, createBid.createBid.descriptor?.effectiveTime)
        val executeMatch = ExecuteMatch.new(
            askId = askUuid.toString(),
            bidId = bidUuid.toString(),
        )
        client.executeContract(signer = deps.accounts.adminAccount, executeMsg = executeMatch)
        client.verifyAskAndBidAreDeleted(askUuid, bidUuid)
        println("Successfully traded coins")
    }
}
