//! DAG-CBOR structured data example
//!
//! This example demonstrates:
//! - Encoding structured data with DAG-CBOR
//! - Storing complex nested structures
//! - Retrieving and decoding data
//! - Working with custom types

use rust_helia::create_helia;
use helia_dag_cbor::{DagCbor, DagCborInterface};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Person {
    name: String,
    age: u32,
    email: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct BlogPost {
    title: String,
    author: Person,
    content: String,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Project {
    name: String,
    description: String,
    contributors: Vec<Person>,
    version: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DAG-CBOR Structured Data Example ===\n");

    let helia = Arc::new(create_helia(None).await?);
    helia.start().await?;
    
    let dag = DagCbor::new(helia.clone());

    // 1. Store a simple structure
    println!("1. Storing a person record...");
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };
    
    let person_cid = dag.add(&person, None).await?;
    println!("   ✓ Person CID: {}\n", person_cid);

    // 2. Retrieve and verify
    println!("2. Retrieving person record...");
    let retrieved_person: Person = dag.get(&person_cid, None).await?;
    println!("   ✓ Name: {}", retrieved_person.name);
    println!("   ✓ Age: {}", retrieved_person.age);
    println!("   ✓ Email: {}\n", retrieved_person.email);
    assert_eq!(person, retrieved_person);

    // 3. Store a complex nested structure
    println!("3. Storing a blog post with nested data...");
    let mut metadata = HashMap::new();
    metadata.insert("published".to_string(), "2024-01-15".to_string());
    metadata.insert("category".to_string(), "Technology".to_string());
    
    let blog_post = BlogPost {
        title: "Introduction to IPFS".to_string(),
        author: person.clone(),
        content: "IPFS is a distributed file system...".to_string(),
        tags: vec![
            "ipfs".to_string(),
            "web3".to_string(),
            "decentralization".to_string(),
        ],
        metadata,
    };
    
    let post_cid = dag.add(&blog_post, None).await?;
    println!("   ✓ Blog post CID: {}\n", post_cid);

    // 4. Retrieve and display the blog post
    println!("4. Retrieving blog post...");
    let retrieved_post: BlogPost = dag.get(&post_cid, None).await?;
    println!("   ✓ Title: {}", retrieved_post.title);
    println!("   ✓ Author: {}", retrieved_post.author.name);
    println!("   ✓ Tags: {}", retrieved_post.tags.join(", "));
    println!("   ✓ Metadata entries: {}\n", retrieved_post.metadata.len());

    // 5. Store a project with multiple contributors
    println!("5. Storing a project with multiple contributors...");
    let contributors = vec![
        Person {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        },
        Person {
            name: "Bob".to_string(),
            age: 28,
            email: "bob@example.com".to_string(),
        },
        Person {
            name: "Charlie".to_string(),
            age: 35,
            email: "charlie@example.com".to_string(),
        },
    ];
    
    let project = Project {
        name: "Helia Rust".to_string(),
        description: "A Rust implementation of Helia IPFS".to_string(),
        contributors,
        version: "0.1.0".to_string(),
    };
    
    let project_cid = dag.add(&project, None).await?;
    println!("   ✓ Project CID: {}\n", project_cid);

    // 6. Retrieve and display project
    println!("6. Retrieving project...");
    let retrieved_project: Project = dag.get(&project_cid, None).await?;
    println!("   ✓ Project: {}", retrieved_project.name);
    println!("   ✓ Version: {}", retrieved_project.version);
    println!("   ✓ Contributors:");
    for contributor in &retrieved_project.contributors {
        println!("     - {} ({})", contributor.name, contributor.email);
    }
    println!();

    helia.stop().await?;
    println!("Example completed successfully!");
    
    Ok(())
}
