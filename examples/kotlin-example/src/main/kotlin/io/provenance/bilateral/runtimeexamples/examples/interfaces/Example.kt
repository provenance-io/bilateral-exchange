package io.provenance.bilateral.runtimeexamples.examples.interfaces

import io.provenance.bilateral.runtimeexamples.config.AppDependencies

interface Example {
    fun start(deps: AppDependencies)
}
