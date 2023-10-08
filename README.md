# vmx

I find the execution model of the Ethereum virtual machine quite interesting.  Any agent can create a program and submit it to the collective machine, where it executes and can interact with other existing programs "smart contracts."

In this project, I played around with the idea of a central matching engine where participants could submit bytecode programs for execution based on certain triggers, and with book state as input.  This has the possible benefit to participants that some more complicated tactics would be viable without low latency connections, as all bytecode programs would execute with equal priority within the engine.

## Status

Small proof of concept, no higher-level language, and matching engine policies are underbaked.

## Related

Check out https://www.onechronos.com for a similar idea but much more mature and with much more thought into the matching mechanisms.
