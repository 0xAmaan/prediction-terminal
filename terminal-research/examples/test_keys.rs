use terminal_research::{ExaClient, OpenAIClient};

fn main() {
    println!("Testing API key validation...\n");

    // Test Exa client
    print!("1. ExaClient::new() - ");
    match ExaClient::new() {
        Ok(_) => println!("SUCCESS (EXA_API_KEY is set)"),
        Err(e) => println!("EXPECTED ERROR: {}", e),
    }

    // Test OpenAI client
    print!("2. OpenAIClient::new() - ");
    match OpenAIClient::new() {
        Ok(_) => println!("SUCCESS (client created, OPENAI_API_KEY will be validated on first request)"),
        Err(e) => println!("ERROR: {}", e),
    }

    println!("\n--- Summary ---");
    println!("If EXA_API_KEY is NOT set: Step 1 should show 'EXPECTED ERROR'");
    println!("If EXA_API_KEY IS set: Step 1 should show 'SUCCESS'");
    println!("Step 2 should always succeed (OpenAI validates on request, not construction)");
}
