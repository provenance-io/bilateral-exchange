package io.provenance.bilateral.runtimeexamples.config

import io.provenance.bilateral.client.BilateralContractClient
import io.provenance.bilateral.runtimeexamples.accounts.BilateralAccounts
import io.provenance.client.grpc.GasEstimationMethod
import io.provenance.client.grpc.PbClient

data class AppDependencies(
    val config: LocalConfig,
    val client: BilateralContractClient,
    val accounts: BilateralAccounts,
) {
    companion object {
        fun load(): AppDependencies = LocalConfig.load().let { config ->
            AppDependencies(
                config = config,
                client = BilateralContractClient(
                    pbClient = PbClient(
                        chainId = config.provenanceConfig.chainId,
                        channelUri = config.provenanceConfig.getChannelUri(),
                        gasEstimationMethod = GasEstimationMethod.MSG_FEE_CALCULATION,
                    ),
                    objectMapper = Singletons.OBJECT_MAPPER,
                    contractName = config.contractConfig.bilateralContractName,
                ),
                accounts = BilateralAccounts.fromConfig(config),
            )
        }
    }
}
