use anyhow::anyhow;

use clap::{App, Arg};

use rusoto_core::region::Region;

use rusoto_sqs::{
	DeleteMessageBatchRequest, DeleteMessageBatchRequestEntry, GetQueueUrlRequest,
	ReceiveMessageRequest, Sqs, SqsClient,
};

use tokio::runtime::Runtime;

use void::Void;

fn main() {
	let matches = App::new("Trivial SQS Consumer")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Consumes SQS messages and prints them")
        .arg(Arg::with_name("region")
             .long("region")
             .help("AWS Region to connect to")
             .takes_value(true)
             .default_value("us-east-1")
             .validator(|name| match name.parse::<Region>() {
                 Ok(_) => Ok(()),
                 Err(e) => Err(e.to_string()),
             }))
        .arg(Arg::with_name("queue-name")
             .long("queue-name")
             .takes_value(true)
             .required(true)
             .help("Queue name to read from"))
        .arg(Arg::with_name("queue-owner")
             .long("queue-owner")
             .takes_value(true)
             .help("AWS Account ID which created the queue, if different from the principal's account"))
    	.arg(Arg::with_name("custom-endpoint")
        	 .long("custom-endpoint")
        	 .takes_value(true)
        	.help("Custom endpoint for AWS connections."))
        .get_matches();

	let region = match matches.value_of("custom-endpoint") {
		None => matches
			.value_of("region")
			.expect("unwrapping argument with default")
			.parse()
			.expect("parsing already-validated input"),
		Some(endpoint) => Region::Custom {
			name: matches
				.value_of("region")
				.expect("unwrapping argument with default")
				.to_owned(),
			endpoint: endpoint.to_owned(),
		},
	};

	let queue_details = GetQueueUrlRequest {
		queue_name: matches
			.value_of("queue-name")
			.expect("unwrapping required argument")
			.to_owned(),
		queue_owner_aws_account_id: matches.value_of("queue-owner").map(str::to_owned),
	};

	let sqs = SqsClient::new(region);

	let mut rt = Runtime::new().expect("spawning runtime");

	match rt.block_on(consume(sqs, queue_details)) {
		Ok(_) => unreachable!(),
		Err(e) => {
			eprintln!("{}", e);
			std::process::exit(1);
		}
	}
}

async fn consume<C: Sqs>(sqs: C, queue: GetQueueUrlRequest) -> anyhow::Result<Void> {
	let queue_url = sqs
		.get_queue_url(queue)
		.await?
		.queue_url
		.ok_or(anyhow!("returned queue URL was empty"))?;

	let message_request = ReceiveMessageRequest {
		queue_url: queue_url.clone(),
		max_number_of_messages: Some(10),
		wait_time_seconds: Some(20),
		..Default::default()
	};

	loop {
		let messages = match sqs.receive_message(message_request.clone()).await?.messages {
			Some(ms) => ms,
			None => continue,
		};

		let mut delete_handles = Vec::with_capacity(messages.len());

		for (i, message) in messages.into_iter().enumerate() {
			if let Some(body) = message.body {
				println!("{:?}", body);
			}
			if let Some(handle) = message.receipt_handle {
				delete_handles.push(DeleteMessageBatchRequestEntry {
					id: i.to_string(),
					receipt_handle: handle,
				});
			}
		}

		sqs.delete_message_batch(DeleteMessageBatchRequest {
			queue_url: queue_url.clone(),
			entries: delete_handles,
		})
		.await?; // N.B.: A partial failure will go unnoticed.
	}
}
