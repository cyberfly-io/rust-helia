//! JSON codec example
//!
//! This example demonstrates:
//! - Storing JSON data in IPFS
//! - Retrieving JSON data by CID
//! - Working with structured JSON objects
//! - Serialization and deserialization

use helia_interface::Helia;
use helia_json::{Json, JsonInterface};
use rust_helia::create_helia;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Example data structures
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
    name: String,
    age: u32,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct BlogPost {
    title: String,
    author: String,
    content: String,
    tags: Vec<String>,
    likes: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== JSON Codec Example ===\n");

    // Create and start Helia node
    let helia = create_helia(None).await?;
    helia.start().await?;

    // Create JSON codec instance
    let json = Json::new(Arc::new(helia));

    // 1. Store a simple Person object
    println!("1. Storing a Person object...");
    let person = Person {
        name: "Alice Smith".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    let person_cid = json.add(&person, None).await?;
    println!("   ✓ Stored Person: {}", person_cid);
    println!("   Data: {:?}\n", person);

    // 2. Retrieve the Person object
    println!("2. Retrieving the Person object...");
    let retrieved_person: Person = json.get(&person_cid, None).await?;
    println!("   ✓ Retrieved Person: {:?}", retrieved_person);
    assert_eq!(person, retrieved_person);
    println!("   ✓ Data matches original\n");

    // 3. Store a BlogPost object
    println!("3. Storing a BlogPost object...");
    let blog_post = BlogPost {
        title: "Getting Started with IPFS".to_string(),
        author: "Bob Johnson".to_string(),
        content: "IPFS is a distributed system for storing and accessing files...".to_string(),
        tags: vec![
            "ipfs".to_string(),
            "web3".to_string(),
            "tutorial".to_string(),
        ],
        likes: 42,
    };

    let post_cid = json.add(&blog_post, None).await?;
    println!("   ✓ Stored BlogPost: {}", post_cid);
    println!("   Title: {}", blog_post.title);
    println!("   Tags: {:?}\n", blog_post.tags);

    // 4. Retrieve the BlogPost object
    println!("4. Retrieving the BlogPost object...");
    let retrieved_post: BlogPost = json.get(&post_cid, None).await?;
    println!("   ✓ Retrieved BlogPost");
    println!("   Title: {}", retrieved_post.title);
    println!("   Author: {}", retrieved_post.author);
    println!("   Likes: {}", retrieved_post.likes);
    assert_eq!(blog_post, retrieved_post);
    println!("   ✓ Data matches original\n");

    // 5. Store a Vec of objects
    println!("5. Storing a list of people...");
    let people = vec![
        Person {
            name: "Charlie Brown".to_string(),
            age: 25,
            email: "charlie@example.com".to_string(),
        },
        Person {
            name: "Diana Prince".to_string(),
            age: 28,
            email: "diana@example.com".to_string(),
        },
        Person {
            name: "Eve Wilson".to_string(),
            age: 35,
            email: "eve@example.com".to_string(),
        },
    ];

    let people_cid = json.add(&people, None).await?;
    println!("   ✓ Stored {} people: {}", people.len(), people_cid);
    for person in &people {
        println!("     - {} (age {})", person.name, person.age);
    }
    println!();

    // 6. Retrieve the list
    println!("6. Retrieving the list of people...");
    let retrieved_people: Vec<Person> = json.get(&people_cid, None).await?;
    println!("   ✓ Retrieved {} people", retrieved_people.len());
    assert_eq!(people, retrieved_people);
    println!("   ✓ All data matches\n");

    // 7. Store nested JSON (using serde_json::Value)
    println!("7. Storing arbitrary JSON data...");
    let json_data = serde_json::json!({
        "project": "Helia Rust",
        "version": "0.1.2",
        "features": ["blockstore", "datastore", "networking"],
        "config": {
            "auto_start": true,
            "max_connections": 100
        }
    });

    let json_cid = json.add(&json_data, None).await?;
    println!("   ✓ Stored JSON: {}", json_cid);
    println!("   Data: {}\n", serde_json::to_string_pretty(&json_data)?);

    // 8. Retrieve arbitrary JSON
    println!("8. Retrieving arbitrary JSON data...");
    let retrieved_json: serde_json::Value = json.get(&json_cid, None).await?;
    println!("   ✓ Retrieved JSON");
    println!("   Project: {}", retrieved_json["project"]);
    println!("   Version: {}", retrieved_json["version"]);
    println!("   Features: {:?}\n", retrieved_json["features"]);

    println!("=== Summary ===");
    println!("✓ Person CID: {}", person_cid);
    println!("✓ BlogPost CID: {}", post_cid);
    println!("✓ People List CID: {}", people_cid);
    println!("✓ JSON Data CID: {}", json_cid);
    println!("\nAll JSON operations completed successfully!");

    Ok(())
}
