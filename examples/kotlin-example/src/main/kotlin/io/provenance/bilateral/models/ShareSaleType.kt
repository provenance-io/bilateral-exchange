package io.provenance.bilateral.models

import com.fasterxml.jackson.annotation.JsonTypeInfo
import com.fasterxml.jackson.databind.PropertyNamingStrategies
import com.fasterxml.jackson.databind.annotation.JsonNaming

@JsonTypeInfo(use = JsonTypeInfo.Id.DEDUCTION)
sealed interface ShareSaleType {
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
    data class SingleTransaction(val singleTransaction: Body) : ShareSaleType {
        @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
        data class Body(val shareCount: String)
    }

    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
    data class MultipleTransactions(val multipleTransactions: Body) : ShareSaleType {
        @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy::class)
        data class Body(val removeSaleShareThreshold: String?)
    }

    companion object {
        fun single(shareCount: String): ShareSaleType = SingleTransaction(SingleTransaction.Body(shareCount))

        fun multiple(removeSaleShareThreshold: String? = null): ShareSaleType =
            MultipleTransactions(MultipleTransactions.Body(removeSaleShareThreshold))
    }
}
