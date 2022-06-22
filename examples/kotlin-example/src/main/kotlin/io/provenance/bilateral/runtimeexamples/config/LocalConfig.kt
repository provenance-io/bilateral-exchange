package io.provenance.bilateral.runtimeexamples.config

import com.akuleshov7.ktoml.Toml
import io.provenance.bilateral.models.AttributeRequirementType
import io.provenance.bilateral.runtimeexamples.examples.enums.ExampleType
import kotlinx.serialization.SerialName
import java.net.URI
import kotlinx.serialization.Serializable

@Serializable
data class LocalConfig(
    val targetExample: String,
    @SerialName("provenance")
    val provenanceConfig: ProvenanceConfig,
    @SerialName("contract")
    val contractConfig: ContractConfig,
    @SerialName("accounts")
    val accountConfig: AccountConfig,
    @SerialName("funding")
    val fundingConfig: FundingConfig,
    @SerialName("coinTrade")
    val coinTradeConfig: CoinTradeConfig,
    @SerialName("markerTrade")
    val markerTradeConfig: MarkerTradeConfig,
    @SerialName("markerShareSaleSingleTx")
    val markerShareSaleSingleTxConfig: MarkerShareSaleSingleTxConfig,
    @SerialName("markerShareSaleMultipleTx")
    val markerShareSaleMultipleTxConfig: MarkerShareSaleMultipleTxConfig,
    @SerialName("requiredAttributes")
    val requiredAttributesConfig: RequiredAttributesConfig,
    @SerialName("scopeTrade")
    val scopeTradeConfig: ScopeTradeConfig,
) {
    companion object {
        fun load(): LocalConfig = Toml()
            .decodeFromString(
                deserializer = serializer(),
                string = ClassLoader.getSystemResource("local.toml").readText()
            )
    }

    fun getTargetExampleEnum(): ExampleType = ExampleType.getByConfigName(targetExample)
}

@Serializable
data class ProvenanceConfig(
    val chainId: String,
    val channelNode: String,
) {
    fun getChannelUri(): URI = URI.create(channelNode)
}

@Serializable
data class ContractConfig(
    @SerialName("name")
    val bilateralContractName: String,
)

@Serializable
data class AccountConfig(
    val askerMnemonic: String,
    val askerHdPath: String,
    val bidderMnemonic: String,
    val bidderHdPath: String,
    val adminMnemonic: String,
    val adminHdPath: String,
    val fundingMnemonic: String,
    val fundingHdPath: String,
)

@Serializable
data class FundingConfig(
    val fundAccountsAtStartup: Boolean,
    val fundingAmount: Long,
    val fundingDenom: String,
)

@Serializable
data class CoinTradeConfig(
    val quoteAmount: Long,
    val quoteDenom: String,
    val baseAmount: Long,
    val baseDenom: String,
)

@Serializable
data class MarkerTradeConfig(
    val markerDenom: String,
)

@Serializable
data class MarkerShareSaleSingleTxConfig(
    val markerDenom: String,
    val shareCount: Long,
    val shareSaleAmount: Long,
)

@Serializable
data class MarkerShareSaleMultipleTxConfig(
    val markerDenom: String,
    val shareCount: Long,
    val sharePurchaseCount: Long,
    val shareCutoff: Long,
)

@Serializable
data class RequiredAttributesConfig(
    val requirementType: String,
    val attributes: List<String>,
) {
    fun getRequirementTypeEnum(): AttributeRequirementType = AttributeRequirementType
        .values()
        .associateBy { it.contractName }
        .let { requirementTypeMap ->
            requirementTypeMap[requirementType]
                ?: throw IllegalArgumentException("Unexpected requirementType value [$requirementType]. Please choose one of: ${requirementTypeMap.keys}")
        }
}

@Serializable
data class ScopeTradeConfig(
    val writeScopeSpec: Boolean,
)
