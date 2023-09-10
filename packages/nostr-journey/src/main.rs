use std::{str::FromStr, time::Duration};

use nostr_journey::PRIVATE_KEY;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let secret_key = SecretKey::from_str(PRIVATE_KEY)?;
    let my_keys = Keys::new(secret_key);

    let message = format!("Hello, nostr! my public key is: {}", my_keys.public_key());

    let client = Client::new(&my_keys);

    client.add_relay("wss://relay.house", None).await?;
    client.add_relay("wss://relay.damus.io", None).await?;
    client.connect().await;

    println!("{message:?}");

    let event_id = client.publish_text_note(message, &[]).await?;
    println!("{:#?}", event_id);

    // Retrieve event by id
    // let filter = Filter::new().id(event_id);

    // Retrieve all the events that we have posted
    let filter = Filter {
        authors: vec![my_keys.public_key().to_string()],
        ..Default::default()
    };
    tokio::time::sleep(Duration::from_secs(2)).await;

    let events = client.get_events_of(vec![filter], None).await?;
    println!("{:#?}", events);

    Ok(())
}
