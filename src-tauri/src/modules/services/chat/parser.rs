use crate::modules::core::types::WhisperEvent;

/// Parse a whisper from a log line
/// Format: "2,From <character> (*<account>): Hi, I'm interested in your Frostburn listed for 2 wss"
/// Format: "4,<character>(<account>) joined our world. Diablo's minions grow stronger."
pub fn parse_whisper(line: &str) -> Option<WhisperEvent> {
    // Check for join messages (starts with "4,")
    if line.starts_with("4,") {
        // Format: "4,shrackx(shrack) joined our world. Diablo's minions grow stronger."
        if line.contains(" joined our world") {
            let after_prefix = &line[2..]; // Skip "4,"
            if let Some(joined_pos) = after_prefix.find(" joined our world") {
                let player_part = after_prefix[..joined_pos].trim();
                // Extract character name and account name
                let (character, account) = if let Some(paren_start) = player_part.find('(') {
                    let character_name = player_part[..paren_start].trim();
                    if let Some(paren_end) = player_part[paren_start..].find(')') {
                        let account_name = player_part[paren_start + 1..paren_start + paren_end].trim();
                        (character_name, account_name)
                    } else {
                        (character_name, "")
                    }
                } else {
                    (player_part, "")
                };
                
                // Use account name if available, otherwise character name
                let sender = if !account.is_empty() {
                    account.strip_prefix('*').unwrap_or(account)
                } else {
                    character
                };
                
                return Some(WhisperEvent {
                    is_trade: false,
                    from: sender.to_string(),
                    message: after_prefix.to_string(),
                    item_name: None,
                    is_join: true,
                });
            }
        }
        return None; // Other type 4 messages, ignore
    }

    // Check if it's a whisper (starts with "2,")
    if !line.starts_with("2,") {
        return None; // Not a whisper line, ignore
    }

    // Ignore "Sent to" messages (outgoing whispers)
    if line.contains("Sent to ") {
        return None;
    }

    // Extract the message part after "From"
    let from_start = match line.find("From ") {
        Some(pos) => pos,
        None => return None, // Not a "From" message, ignore
    };
    let after_from = &line[from_start + 5..]; // Skip "From "
    
    // Find the colon that separates sender from message
    let colon_pos = match after_from.find(':') {
        Some(pos) => pos,
        None => return None, // No colon found, malformed line, ignore
    };
    let sender_part = &after_from[..colon_pos].trim();
    let message = after_from[colon_pos + 1..].trim();

    // Ignore friend online/offline messages
    if message.contains("Your friend") && (message.contains("has left Project Diablo 2") || message.contains("has entered Project Diablo 2")) {
        return None;
    }

    // Extract sender name - prefer account name from parentheses, otherwise use character name
    // Format: "shrackx (*shrack)" or "shrackx (*shrack)" or just "shrackx"
    let sender = if let Some(paren_start) = sender_part.find('(') {
        // Extract account name from parentheses (e.g., "*shrack" from "(*shrack)")
        if let Some(paren_end) = sender_part[paren_start..].find(')') {
            let account_name = &sender_part[paren_start + 1..paren_start + paren_end].trim();
            // Remove "*" prefix if present
            account_name.strip_prefix('*').unwrap_or(account_name)
        } else {
            // Fallback to character name if parentheses are malformed
            sender_part.split_whitespace().next().unwrap_or(sender_part)
        }
    } else if let Some(space_pos) = sender_part.find(' ') {
        &sender_part[..space_pos]
    } else {
        sender_part
    };

    // Check if it's a trade whisper
    let is_trade = message.starts_with("Hi, I'm interested in your");
    
    // Extract item name from trade whisper
    let item_name = if is_trade {
        // Format: "Hi, I'm interested in your Frostburn listed for 2 wss"
        // Extract item name between "your" and "listed"
        if let Some(your_pos) = message.find("your ") {
            let after_your = &message[your_pos + 5..];
            if let Some(listed_pos) = after_your.find(" listed") {
                Some(after_your[..listed_pos].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    Some(WhisperEvent {
        is_trade,
        from: sender.to_string(),
        message: message.to_string(),
        item_name,
        is_join: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trade_whisper() {
        let line = "2,From Character (*Account): Hi, I'm interested in your Item Name listed for 1 hr";
        let event = parse_whisper(line).unwrap();
        assert!(event.is_trade);
        assert_eq!(event.from, "Account");
        assert_eq!(event.item_name, Some("Item Name".to_string()));
    }

    #[test]
    fn test_parse_join_message() {
        let line = "4,Character(Account) joined our world. Diablo's minions grow stronger.";
        let event = parse_whisper(line).unwrap();
        assert!(event.is_join);
        assert_eq!(event.from, "Account");
    }
}
