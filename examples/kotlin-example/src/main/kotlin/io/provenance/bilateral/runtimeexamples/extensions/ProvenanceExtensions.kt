package io.provenance.bilateral.runtimeexamples.extensions

import cosmos.tx.v1beta1.ServiceOuterClass.BroadcastTxResponse

fun BroadcastTxResponse.checkZeroResponseCode(): BroadcastTxResponse = this.also {
    check(this.txResponse.code == 0) { "Received non-zero response code!  Error message: ${this.txResponse.rawLog}" }
}
