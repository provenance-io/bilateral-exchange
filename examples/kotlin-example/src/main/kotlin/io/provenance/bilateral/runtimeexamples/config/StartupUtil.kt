package io.provenance.bilateral.runtimeexamples.config

import cosmos.bank.v1beta1.Tx
import cosmos.tx.v1beta1.ServiceOuterClass
import io.provenance.bilateral.runtimeexamples.extensions.checkZeroResponseCode
import io.provenance.bilateral.runtimeexamples.functions.coin
import io.provenance.client.grpc.BaseReqSigner
import io.provenance.client.protobuf.extensions.toAny
import io.provenance.client.protobuf.extensions.toTxBody
import org.bouncycastle.jce.provider.BouncyCastleProvider
import java.security.Security
import java.util.TimeZone

object StartupUtil {
    fun setupEnvironmentLoadDeps(): AppDependencies {
        TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
        Security.addProvider(BouncyCastleProvider())
        return AppDependencies.load().also { deps ->
            if (deps.config.fundingConfig.fundAccountsAtStartup) {
                fundAccounts(deps)
            }
        }
    }

    private fun fundAccounts(deps: AppDependencies) {
        val messages = listOf(deps.accounts.askerAccount, deps.accounts.bidderAccount, deps.accounts.adminAccount)
            .map { account ->
                Tx.MsgSend.newBuilder().also { send ->
                    send.fromAddress = deps.accounts.fundingAccount.address()
                    send.toAddress = account.address()
                    send.addAmount(coin(deps.config.fundingConfig.fundingAmount, deps.config.fundingConfig.fundingDenom))
                }.build().toAny()
            }
        println("Funding all accounts...")
        deps.client.pbClient.estimateAndBroadcastTx(
            txBody = messages.toTxBody(),
            signers = BaseReqSigner(deps.accounts.fundingAccount).let(::listOf),
            mode = ServiceOuterClass.BroadcastMode.BROADCAST_MODE_BLOCK,
            gasAdjustment = 1.3,
        ).checkZeroResponseCode().also {
            println("Successfully funded all accounts with ${deps.config.fundingConfig.fundingAmount}${deps.config.fundingConfig.fundingDenom}")
        }
    }
}
