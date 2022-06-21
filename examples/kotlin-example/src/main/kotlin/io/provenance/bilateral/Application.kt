package io.provenance.bilateral

import io.provenance.bilateral.runtimeexamples.config.StartupUtil

fun main() {
    println("Thank you for choosing the Bilateral Exchange Smart Contract for your example needs.  I hope you have a nice stay... without Exceptions.")
    StartupUtil.setupEnvironmentLoadDeps().also { deps ->
        deps.config.getTargetExampleEnum().example.also { example ->
            println("Running example of type [${example::class.simpleName}]")
        }.start(deps)
    }
}
