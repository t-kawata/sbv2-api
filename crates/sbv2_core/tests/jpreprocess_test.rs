use sbv2_core::jtalk::JTalk;
use sbv2_core::tts_util::preprocess_parse_text;

#[test]
fn test_jpreprocess_with_various_texts() {
    let jtalk = JTalk::new().expect("Failed to initialize JTalk");

    let test_texts = vec![
        "お電話",
        "お名前",
        "お日にち",
        "お手前",
        "こんにちは",
        "今日は良い天気です",
    ];

    for text in test_texts {
        println!("Testing: {}", text);

        match preprocess_parse_text(text, &jtalk) {
            Ok((normalized_text, process)) => {
                println!("Preprocessing success: {}", normalized_text);
                match process.g2p() {
                    Ok(_) => println!("Full pipeline success"),
                    Err(e) => println!("g2p failed: {}", e),
                }
            }
            Err(e) => println!("Preprocessing failed: {}", e),
        }
    }
}
