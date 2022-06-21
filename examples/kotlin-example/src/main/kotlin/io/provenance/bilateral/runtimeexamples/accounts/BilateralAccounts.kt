package io.provenance.bilateral.runtimeexamples.accounts

import io.provenance.bilateral.runtimeexamples.config.LocalConfig


data class BilateralAccounts(
    val askerAccount: MnemonicSigner,
    val bidderAccount: MnemonicSigner,
    val adminAccount: MnemonicSigner,
    val fundingAccount: MnemonicSigner,
) {
    companion object {
        fun fromConfig(
            config: LocalConfig,
            printAddresses: Boolean = true,
        ): BilateralAccounts = config.accountConfig.let { accountConfig ->
            BilateralAccounts(
                askerAccount = MnemonicSigner.new(accountConfig.askerMnemonic, accountConfig.askerHdPath),
                bidderAccount = MnemonicSigner.new(accountConfig.bidderMnemonic, accountConfig.bidderHdPath),
                adminAccount = MnemonicSigner.new(accountConfig.adminMnemonic, accountConfig.adminHdPath),
                fundingAccount = MnemonicSigner.new(accountConfig.fundingMnemonic, accountConfig.fundingHdPath),
            )
        }.also { accounts ->
            if (printAddresses) {
                println("""
                    Using asker account with address:   [${accounts.askerAccount.address()}]
                    Using bidder account with address:  [${accounts.bidderAccount.address()}]
                    Using admin account with address:   [${accounts.adminAccount.address()}]
                    Using funding account with address: [${accounts.fundingAccount.address()}]
                """.trimIndent())
            }
        }
    }
}
