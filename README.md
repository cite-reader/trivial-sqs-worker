# A Trivial SQS Worker

Seriously, I mean absolutely trivial. All it does is print out the contents of messages it receives.

## How exactly does this work?

The main function is mostly taken up with argument parsing goop, so if you're following along for the SQS bits you can pretty much ignore it. The meat is the function `consume`: an infinite loop of [long polls](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-short-and-long-polling.html#sqs-long-polling) that prints out everything it got before deleting the batch.

Note that this pattern is optimized (if you want to call it that) for _very quick_ message processing; in this case it's pretty much the simplest interesting thing we could possibly be doing, and the network delay is going to absolutely swamp anything else. If your messages take longer to process, like say processing high-definition video files, you may want to accept fewer messages at a time and/or delete them as each one finishes. You know your workload better than me, I assume. Note also that error handling here is primitive for the sake of banging out a quick example, and in particular a _partial failure_ of the batch delete will go entirely unnoticed. Real code will want to check for that and I guess retry the delete a few times?

## Can I use this?

It's really simple code, so don't worry about it. If you need a formal license, the code may be used under the terms of the [Blue Oak Model License 1.0.0](https://blueoakcouncil.org/license/1.0.0) as specified in the Cargo.toml file.
