package io.provenance.bilateral.runtimeexamples.extensions

import io.provenance.bilateral.client.BilateralContractClient
import io.provenance.bilateral.runtimeexamples.config.Singletons
import java.time.OffsetDateTime
import java.util.UUID

fun BilateralContractClient.checkAsk(askUuid: UUID, expectedTime: OffsetDateTime? = null) {
    this.getAsk(askUuid.toString()).also { ask ->
        println("Found ask: ${Singletons.OBJECT_MAPPER.writerWithDefaultPrettyPrinter().writeValueAsString(ask)}")
        expectedTime?.takeIf { ask.descriptor?.effectiveTime != null }?.also { time ->
            check(time == ask.descriptor?.effectiveTime) { "Mismatched times.  Original: [$expectedTime], Contract Version: [${ask.descriptor?.effectiveTime}]" }
        }
    }
}

fun BilateralContractClient.checkBid(bidUuid: UUID, expectedTime: OffsetDateTime? = null) {
    this.getBid(bidUuid.toString()).also { bid ->
        println("Found bid: ${Singletons.OBJECT_MAPPER.writerWithDefaultPrettyPrinter().writeValueAsString(bid)}")
        expectedTime?.takeIf { bid.descriptor?.effectiveTime != null }?.also { time ->
            check(time == bid.descriptor?.effectiveTime) { "Mismatched times.  Original: [$expectedTime], Contract Version: [${bid.descriptor?.effectiveTime}]" }
        }
    }
}

fun BilateralContractClient.verifyAskAndBidAreDeleted(
    askUuid: UUID,
    bidUuid: UUID,
    verifyAsk: Boolean = true,
    verifyBid: Boolean = true,
) {
    if (verifyAsk) {
        try {
            this.getAsk(askUuid.toString())
        } catch (e: Exception) {
            println("Ask is missing from storage as expected")
            null
        }.also {
            if (it != null) {
                throw IllegalStateException("Ask should be missing!!!")
            }
        }
    }
    if (!verifyBid) {
        return
    }
    try {
        this.getBid(bidUuid.toString())
    } catch (e: Exception) {
        println("Bid is missing from storage as expected")
        null
    }.also {
        if (it != null) {
            throw IllegalStateException("Bid should be missing!!!")
        }
    }
}
