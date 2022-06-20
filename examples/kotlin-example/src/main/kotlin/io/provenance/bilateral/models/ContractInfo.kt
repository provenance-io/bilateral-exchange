package io.provenance.bilateral.models

import com.fasterxml.jackson.databind.PropertyNamingStrategies.SnakeCaseStrategy
import com.fasterxml.jackson.databind.annotation.JsonNaming

@JsonNaming(SnakeCaseStrategy::class)
data class ContractInfo(
    val admin: String,
    val bindName: String,
    val contractName: String,
    val contractType: String,
    val contractVersion: String,
)
