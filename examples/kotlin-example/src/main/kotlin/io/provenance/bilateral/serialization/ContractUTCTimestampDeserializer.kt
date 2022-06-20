package io.provenance.bilateral.serialization

import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.databind.DeserializationContext
import com.fasterxml.jackson.databind.JsonDeserializer
import java.math.BigDecimal
import java.math.RoundingMode
import java.time.Instant
import java.time.OffsetDateTime
import java.time.ZoneOffset

/**
 * See ContractUTCTimestampSerializer for information on why this is necessary for cosmwasm Timestamp struct values.
 */
class ContractUTCTimestampDeserializer : JsonDeserializer<OffsetDateTime>() {
    private companion object {
        private val NANO_TO_SECONDS_DIVISOR = BigDecimal.valueOf(1_000_000_000L)
    }

    override fun deserialize(p: JsonParser?, ctxt: DeserializationContext?): OffsetDateTime? = p
        ?.text
        ?.trim()
        ?.toBigDecimal()
        ?.let { epochNanos ->
            val nanos = epochNanos % NANO_TO_SECONDS_DIVISOR
            // Subtract the remainder nanos from the epoch nanos to ensure that division is perfect and no rounding
            // discrepancies can occur
            val epochSeconds = epochNanos.minus(nanos).divide(NANO_TO_SECONDS_DIVISOR, RoundingMode.UNNECESSARY)
            OffsetDateTime.ofInstant(Instant.ofEpochSecond(epochSeconds.toLong(), nanos.toLong()), ZoneOffset.UTC)
        }
}
