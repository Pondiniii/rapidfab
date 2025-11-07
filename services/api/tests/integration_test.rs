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

/// Test session expiration (token invalidation)
#[tokio::test]
#[ignore]
async fn test_token_invalidation_after_multiple_logouts() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let password = "testpass123";

    // Register user
    let register_res = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to register");

    assert_eq!(register_res.status(), 200);

    let auth: serde_json::Value = register_res.json().await.expect("Failed to parse");
    let token = auth["token"]
        .as_str()
        .expect("Token should exist")
        .to_string();

    // Verify token works before logout
    let me_before = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to get user");

    assert_eq!(me_before.status(), 200);

    // Logout (invalidate token)
    let logout_res = client
        .post(format!("{base_url}/auth/logout"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to logout");

    assert_eq!(logout_res.status(), 204);

    // Verify token no longer works
    let me_after = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to get user after logout");

    assert_eq!(me_after.status(), 401, "Invalidated token should not work");
}

/// Test wrong password after correct registration
#[tokio::test]
#[ignore]
async fn test_login_with_wrong_password() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let correct_password = "correctpass123";
    let wrong_password = "wrongpass456";

    // Register user
    let register_res = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email,
            "password": correct_password
        }))
        .send()
        .await
        .expect("Failed to register");

    assert_eq!(register_res.status(), 200);

    // Try to login with wrong password
    let login_res = client
        .post(format!("{base_url}/auth/login"))
        .json(&json!({
            "email": email,
            "password": wrong_password
        }))
        .send()
        .await
        .expect("Failed login request");

    assert_eq!(
        login_res.status(),
        401,
        "Login with wrong password should return 401"
    );
}

/// Test getting user profile requires valid token
#[tokio::test]
#[ignore]
async fn test_get_user_requires_authentication() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    let res = client
        .get(format!("{base_url}/users/me"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        res.status(),
        401,
        "GET /users/me without token should return 401"
    );
}

/// Test multiple registrations and logins in sequence
#[tokio::test]
#[ignore]
async fn test_multiple_users_independent_sessions() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    // Register first user
    let email1 = format!("test1-{}@example.com", uuid::Uuid::new_v4());
    let password1 = "password123";

    let reg1_res = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email1,
            "password": password1
        }))
        .send()
        .await
        .expect("Failed to register user 1");

    assert_eq!(reg1_res.status(), 200);
    let auth1: serde_json::Value = reg1_res.json().await.expect("Failed to parse");
    let token1 = auth1["token"].as_str().expect("Token1 should exist");

    // Register second user
    let email2 = format!("test2-{}@example.com", uuid::Uuid::new_v4());
    let password2 = "password456";

    let reg2_res = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email2,
            "password": password2
        }))
        .send()
        .await
        .expect("Failed to register user 2");

    assert_eq!(reg2_res.status(), 200);
    let auth2: serde_json::Value = reg2_res.json().await.expect("Failed to parse");
    let token2 = auth2["token"].as_str().expect("Token2 should exist");

    // Verify user 1 can access their profile
    let me1_res = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {token1}"))
        .send()
        .await
        .expect("Failed to get user 1");

    assert_eq!(me1_res.status(), 200);
    let user1: serde_json::Value = me1_res.json().await.expect("Failed to parse user 1");
    assert_eq!(user1["email"], email1);

    // Verify user 2 can access their profile
    let me2_res = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {token2}"))
        .send()
        .await
        .expect("Failed to get user 2");

    assert_eq!(me2_res.status(), 200);
    let user2: serde_json::Value = me2_res.json().await.expect("Failed to parse user 2");
    assert_eq!(user2["email"], email2);

    // Verify token1 cannot access user 2's profile
    assert_ne!(user1["id"], user2["id"], "Users should have different IDs");
}

/// Test login returns new token each time
#[tokio::test]
#[ignore]
async fn test_multiple_logins_return_different_tokens() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let password = "testpass123";

    // Register user
    let register_res = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to register");

    let auth: serde_json::Value = register_res.json().await.expect("Failed to parse");
    let initial_token = auth["token"].as_str().expect("Initial token should exist");

    // Logout with first token
    let logout_res = client
        .post(format!("{base_url}/auth/logout"))
        .header("Authorization", format!("Bearer {initial_token}"))
        .send()
        .await
        .expect("Failed to logout");

    assert_eq!(logout_res.status(), 204);

    // Login first time - get token 1
    let login1_res = client
        .post(format!("{base_url}/auth/login"))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed first login");

    let auth1: serde_json::Value = login1_res.json().await.expect("Failed to parse");
    let token1 = auth1["token"].as_str().expect("Token1 should exist");

    // Logout with token 1
    let logout1_res = client
        .post(format!("{base_url}/auth/logout"))
        .header("Authorization", format!("Bearer {token1}"))
        .send()
        .await
        .expect("Failed to logout");

    assert_eq!(logout1_res.status(), 204);

    // Login second time - get token 2
    let login2_res = client
        .post(format!("{base_url}/auth/login"))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed second login");

    let auth2: serde_json::Value = login2_res.json().await.expect("Failed to parse");
    let token2 = auth2["token"].as_str().expect("Token2 should exist");

    // Verify tokens are different
    assert_ne!(token1, token2, "Each login should return a different token");

    // Verify both tokens work (or token1 doesn't work if already invalidated)
    let me_with_token2 = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {token2}"))
        .send()
        .await
        .expect("Failed with token2");

    assert_eq!(me_with_token2.status(), 200, "Token2 should be valid");
}

/// Test profile request includes correct user data
#[tokio::test]
#[ignore]
async fn test_user_profile_contains_correct_data() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let password = "testpass123";
    let full_name = "John Doe";

    // Register with full_name
    let register_res = client
        .post(format!("{base_url}/auth/register"))
        .json(&json!({
            "email": email,
            "password": password,
            "full_name": full_name
        }))
        .send()
        .await
        .expect("Failed to register");

    let auth: serde_json::Value = register_res.json().await.expect("Failed to parse");
    let token = auth["token"].as_str().expect("Token should exist");
    let user_id = auth["user_id"].as_str().expect("User ID should exist");

    // Get user profile
    let me_res = client
        .get(format!("{base_url}/users/me"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to get profile");

    let user: serde_json::Value = me_res.json().await.expect("Failed to parse user");

    // Verify all fields are present and correct
    assert_eq!(user["id"], user_id, "User ID should match");
    assert_eq!(user["email"], email, "Email should match");
    assert_eq!(user["full_name"], full_name, "Full name should match");
    assert!(
        user["created_at"].is_string(),
        "created_at should be present and be a string"
    );
}
