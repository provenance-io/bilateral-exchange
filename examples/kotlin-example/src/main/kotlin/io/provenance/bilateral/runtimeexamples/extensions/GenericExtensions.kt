package io.provenance.bilateral.runtimeexamples.extensions

fun <T> T.wrapList(): List<T> = listOf(this)
