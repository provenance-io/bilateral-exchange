package io.provenance.bilateral.query

import com.fasterxml.jackson.annotation.JsonValue
import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming
import io.provenance.bilateral.interfaces.ContractQueryMsg

@JsonNaming(SnakeCaseStrategy::class)
data class ContractSearchRequest(
    val searchType: ContractSearchType,
    val pageSize: Int? = null,
    val pageNumber: Int? = null,
) : ContractQueryMsg

sealed interface ContractSearchType {
    @JsonNaming(SnakeCaseStrategy::class)
    object All : ContractSearchType {
        @JsonValue
        fun serializeAs(): String = "all"
    }

    @JsonNaming(SnakeCaseStrategy::class)
    data class Type(val valueType: Body) : ContractSearchType {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val valueType: String)
    }

    @JsonNaming(SnakeCaseStrategy::class)
    data class Id(val id: Body) : ContractSearchType {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val id: String)
    }

    @JsonNaming(SnakeCaseStrategy::class)
    data class Owner(val owner: Body) : ContractSearchType {
        @JsonNaming(SnakeCaseStrategy::class)
        data class Body(val owner: String)
    }

    companion object {
        fun all(): ContractSearchType = All
        fun byType(type: String): ContractSearchType = Type(Type.Body(type))
        fun byId(id: String): ContractSearchType = Id(Id.Body(id))
        fun byOwner(owner: String): ContractSearchType = Owner(Owner.Body(owner))
    }
}

@JsonNaming(SnakeCaseStrategy::class)
data class ContractSearchResult<T>(
    val results: List<T>,
    val pageNumber: Int,
    val pageSize: Int,
    val totalPages: Int,
)

