use haxnews_core::feed::parser::FeedParser;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Example RSS feed XML
    let rss_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>The Hacker News</title>
    <link>https://thehackernews.com</link>
    <description>Most trusted, widely-read independent cybersecurity news source for everyone</description>
    <language>en-us</language>
    <lastBuildDate>Sun, 31 May 2026 14:49:27 +0530</lastBuildDate>
    <item>
      <title>PAN-OS GlobalProtect Authentication Bypass (CVE-2026-0257) Under Active Exploitation</title>
      <description>Critical vulnerability in Palo Alto Networks PAN-OS</description>
      <link>https://thehackernews.com/2026/05/pan-os-globalprotect-authentication.html</link>
      <guid isPermaLink="false">https://thehackernews.com/2026/05/pan-os-globalprotect-authentication.html</guid>
      <pubDate>Sat, 30 May 2026 12:11:26 +0530</pubDate>
      <author>info@thehackernews.com (The Hacker News)</author>
      <enclosure length="12216320" type="image/jpeg" url="https://blogger.googleusercontent.com/img/b/R29vZ2xl/AVvXsEgkaW0i4ALAlpWQ_cOjfhoqUlNgMlZysJA6ay0qPViGI_KxEEG-Hh0KdtWLqBXDH42ZBGSONs0ZJuzOqdRF7vbx6Xa9J8HlP60lY45JHy0ivdRQs0exe4wZT2lI3TW4oDO-XXPVz2pek2M3izLqT3ONwq2iuHPN31ZZvK3jl0zIDq_h5XF1CTRk7fUPzjEQ/s1600/panos.jpg"/>
    </item>
    <item>
      <title>ChatGPhish Vulnerability Turns ChatGPT Web Summaries Into a Phishing Surface</title>
      <description>Cybersecurity researchers have disclosed details of a vulnerability in OpenAI ChatGPT that leverages the artificial intelligence (AI) assistant's implicit trust in Markdown links and images to trigger prompt injections and open the door to phishing attacks.</description>
      <link>https://thehackernews.com/2026/05/chatgphish-vulnerability-turns-chatgpt.html</link>
      <guid isPermaLink="false">https://thehackernews.com/2026/05/chatgphish-vulnerability-turns-chatgpt.html</guid>
      <pubDate>Fri, 29 May 2026 23:37:12 +0530</pubDate>
      <author>info@thehackernews.com (The Hacker News)</author>
      <enclosure length="12216320" type="image/jpeg" url="https://blogger.googleusercontent.com/img/b/R29vZ2xl/AVvXsEikkk-MbHPjc5UpAORUC9pUfe-LntIu7A2tsg3EBFPXh3b6WXoiv8HtxvSakdqICfwN1YGSY452zIdjuyafscYfbf7yKnzbE_SxWxmPeX9uBLkTWY7aNyzLK903ts83ThlQGKOPYKNCW6UHg2c7ia4O7cVIwV5p24c-POfHYTJak6tRmL03rbjOWxCfpPYb/s1600/chatgpt-phishing.jpg"/>
    </item>
    <item>
      <title>Attackers Use LLM Agent for Post-Exploitation After Marimo CVE-2026-39987 Exploit</title>
      <description>An unknown threat actor has been observed using a large language model (LLM) agent to conduct post-compromise actions after obtaining initial access following the exploitation of a publicly-accessible Marimo network.</description>
      <link>https://thehackernews.com/2026/05/attackers-use-llm-agent-for-post.html</link>
      <guid isPermaLink="false">https://thehackernews.com/2026/05/attackers-use-llm-agent-for-post.html</guid>
      <pubDate>Fri, 29 May 2026 20:09:56 +0530</pubDate>
      <author>info@thehackernews.com (The Hacker News)</author>
      <enclosure length="12216320" type="image/jpeg" url="https://blogger.googleusercontent.com/img/b/R29vZ2xl/AVvXsEi20dgnD8cZh6NCcPM9Xa3fzLgNygU4O6AmBUmN1w6KwsDMJ8_jkpZPk77r8phf3MX-cXOlVxke-ypIuj2xh3AB3dy1HSuIa4YYFlgH8Odm1jCRVESBGqxgiDoRbQEG4L_QrKOoH8TSvLLKZxnBfPEemz4kaqWto4t_3cZCmWW44NX-Q1aWakBWVDhAza7T/s1600/marimo.png"/>
    </item>
  </channel>
</rss>"#;

    println!("🔍 Testing RSS Feed Parser");
    println!("========================\n");

    // Create a new feed ID
    let feed_id = Uuid::new_v4();
    
    // Parse the RSS feed
    match FeedParser::parse(feed_id, rss_content, "The Hacker News") {
        Ok(items) => {
            println!("✅ Successfully parsed {} items\n", items.len());
            
            for (idx, item) in items.iter().enumerate() {
                println!("Item {}:", idx + 1);
                println!("  Title: {}", item.title);
                println!("  Link: {}", item.link);
                println!("  Author: {}", item.author.as_deref().unwrap_or("Unknown"));
                println!("  Published: {}", item.published_at
                    .map(|dt| dt.to_rfc2822())
                    .unwrap_or("Unknown".to_string()));
                println!("  Image URL: {}", item.image_url.as_deref().unwrap_or("None"));
                println!("  Dedup Hash: {}", &item.dedup_hash[..16]); // Show first 16 chars
                println!("  Summary: {}...\n", &item.summary.as_deref().unwrap_or("")[..100.min(item.summary.as_deref().unwrap_or("").len())]);
            }
        }
        Err(e) => {
            eprintln!("❌ Error parsing feed: {}", e);
        }
    }

    Ok(())
}
