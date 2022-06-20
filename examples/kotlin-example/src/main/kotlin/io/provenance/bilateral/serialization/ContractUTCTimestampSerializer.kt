package io.provenance.bilateral.serialization

import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.databind.JsonSerializer
import com.fasterxml.jackson.databind.SerializerProvider
import java.math.BigDecimal
import java.time.OffsetDateTime

/**
 * Converts an OffsetDateTime into its epoch nanosecond value.  The cosmwasm Timestamp struct requires epoch nanos to
 * track time values, and this will ensure time fields are properly structured.
 */
class ContractUTCTimestampSerializer : JsonSerializer<OffsetDateTime>() {
    private companion object {
        private val SECONDS_TO_NANOS_MULTIPLIER = BigDecimal.valueOf(1_000_000_000L)
    }

    override fun serialize(value: OffsetDateTime, gen: JsonGenerator, serializers: SerializerProvider?) {
        value.also { odt ->
            // A Long cannot represent all possible epoch nanos, so we default to BigDecimal to ensure large numbers
            // can be handled
            val epochNanos = odt.toEpochSecond()
                .toBigDecimal()
                .multiply(SECONDS_TO_NANOS_MULTIPLIER)
                .plus(odt.nano.toBigDecimal())
            gen.writeString(epochNanos.toPlainString())
        }
    }
}
