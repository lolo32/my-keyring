package eu.baysse.mykeyring.helper

class RustGreeting {
    private external fun greeting(to: String): String

    fun sayHello(to: String): String {
        return greeting(to);
    }
}
