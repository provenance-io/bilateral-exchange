package io.provenance.bilateral.runtimeexamples.examples.enums

import io.provenance.bilateral.runtimeexamples.examples.CoinTradeExample
import io.provenance.bilateral.runtimeexamples.examples.MarkerShareSaleMultipleTxExample
import io.provenance.bilateral.runtimeexamples.examples.MarkerShareSaleSingleTxExample
import io.provenance.bilateral.runtimeexamples.examples.MarkerTradeExample
import io.provenance.bilateral.runtimeexamples.examples.RequiredAttributesExample
import io.provenance.bilateral.runtimeexamples.examples.ScopeTradeExample
import io.provenance.bilateral.runtimeexamples.examples.SearchContractExample
import io.provenance.bilateral.runtimeexamples.examples.interfaces.Example

enum class ExampleType(val configName: String, val example: Example) {
    COIN_TRADE("coin_trade", CoinTradeExample),
    MARKER_SHARE_SALE_SINGLE_TX("marker_share_sale_single", MarkerShareSaleSingleTxExample),
    MARKER_SHARE_SALE_MULTI_TX("marker_share_sale_multi", MarkerShareSaleMultipleTxExample),
    MARKER_TRADE("marker_trade", MarkerTradeExample),
    REQUIRED_ATTRIBUTES("required_attributes", RequiredAttributesExample),
    SCOPE_TRADE("scope_trade", ScopeTradeExample),
    SEARCH("search", SearchContractExample);

    companion object {
        val configNameMap: Map<String, ExampleType> by lazy { ExampleType.values().associateBy { it.configName } }

        fun getByConfigName(configName: String): ExampleType = configNameMap[configName]
            ?: throw IllegalArgumentException("No example of type [$configName] exists.  Please provide one of: ${configNameMap.keys}")
    }
}
