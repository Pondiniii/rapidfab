use serde_json::json;

/// Integration test for full authentication flow
/// Requires PostgreSQL running on localhost:5432
/// Run with: cargo test --test integration_test
#[tokio::test]
#[ignore] // Run only when explicitly requested
async fn test_auth_flow() {
    // Note: requires docker-compose up -d
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    // Test data
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let password = "testpass123";
    let full_name = "Test User";

    // 1. Register new user
    let register_res = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email,
            "password": password,
            "full_name": full_name
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_eq!(register_res.status(), 200, "Registration should succeed");

    let auth: serde_json::Value = register_res.json().await.expect("Failed to parse JSON");
    let token = auth["token"].as_str().expect("Token should be present");
    let user_id = auth["user_id"].as_str().expect("User ID should be present");

    assert!(!token.is_empty(), "Token should not be empty");
    assert!(!user_id.is_empty(), "User ID should not be empty");

    // 2. Get current user profile
    let me_res = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send /users/me request");

    assert_eq!(me_res.status(), 200, "/users/me should succeed");

    let user: serde_json::Value = me_res.json().await.expect("Failed to parse user JSON");
    assert_eq!(user["email"], email, "Email should match");
    assert_eq!(user["full_name"], full_name, "Full name should match");

    // 3. Logout
    let logout_res = client
        .post(format!("{base_url}/auth/logout"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send logout request");

    assert_eq!(logout_res.status(), 204, "Logout should return 204");

    // 4. Verify token is invalid after logout
    let me_after_logout = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send /users/me after logout");

    assert_eq!(
        me_after_logout.status(),
        401,
        "Should be unauthorized after logout"
    );

    // 5. Login with same credentials
    let login_res = client
        .post(format!("{base_url}/auth/login"))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(login_res.status(), 200, "Login should succeed");

    let login_auth: serde_json::Value = login_res.json().await.expect("Failed to parse login JSON");
    let new_token = login_auth["token"]
        .as_str()
        .expect("New token should be present");

    assert!(!new_token.is_empty(), "New token should not be empty");
    assert_ne!(
        new_token, token,
        "New token should be different from old token"
    );

    // 6. Verify new token works
    let me_with_new_token = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {new_token}"))
        .send()
        .await
        .expect("Failed to send /users/me with new token");

    assert_eq!(
        me_with_new_token.status(),
        200,
        "Should succeed with new token"
    );
}

/// Test registration with duplicate email
#[tokio::test]
#[ignore]
async fn test_duplicate_email() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    let email = format!("duplicate-{}@example.com", uuid::Uuid::new_v4());

    // Register first time
    let first_reg = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email,
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed first registration");

    assert_eq!(first_reg.status(), 200);

    // Try to register again with same email
    let second_reg = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email,
            "password": "password456"
        }))
        .send()
        .await
        .expect("Failed second registration");

    assert_eq!(
        second_reg.status(),
        409,
        "Should return conflict for duplicate email"
    );
}

/// Test invalid credentials
#[tokio::test]
#[ignore]
async fn test_invalid_credentials() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    let login_res = client
        .post(format!("{base_url}/auth/login"))
        .json(&json!({
            "email": "nonexistent@example.com",
            "password": "wrongpassword"
        }))
        .send()
        .await
        .expect("Failed login request");

    assert_eq!(
        login_res.status(),
        401,
        "Should return unauthorized for invalid credentials"
    );
}
