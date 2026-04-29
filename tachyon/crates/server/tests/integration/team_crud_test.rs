use tachyon_database::{Team, TeamRepository};

use crate::common::setup::{create_test_pool, create_test_user, setup_database, teardown_database};

fn skip_without_db() -> bool {
    std::env::var("DATABASE_URL").is_err() && std::env::var("TEST_DATABASE_URL").is_err()
}

fn make_team(owner_id: &str, name: &str) -> Team {
    Team::new(
        name.to_string(),
        name.to_lowercase().replace(' ', "-"),
        owner_id.to_string(),
    )
    .with_description(format!("{} team description", name))
}

#[tokio::test]
async fn test_create_team() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TeamRepository::new(pool.clone());

    let team = make_team(&user.id.as_str(), "Engineering");
    let created = repo.create(&team).await.expect("Failed to create team");

    assert!(!created.id.is_empty());
    assert_eq!(created.name, "Engineering");
    assert_eq!(created.owner_id, user.id.as_str());
    assert!(created.description.is_some());

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_get_team_by_id() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TeamRepository::new(pool.clone());

    let team = make_team(&user.id.as_str(), "Get Test Team");
    let created = repo.create(&team).await.expect("Failed to create team");

    let fetched = repo
        .get_by_id(&created.id)
        .await
        .expect("Failed to get team by ID");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Get Test Team");

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_get_team_by_slug() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TeamRepository::new(pool.clone());

    let team = make_team(&user.id.as_str(), "Slug Team");
    let created = repo.create(&team).await.expect("Failed to create team");

    let fetched = repo
        .get_by_slug(&created.slug)
        .await
        .expect("Failed to get team by slug");
    assert_eq!(fetched.id, created.id);

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_list_teams() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TeamRepository::new(pool.clone());

    repo.create(&make_team(&user.id.as_str(), "Team Alpha"))
        .await
        .expect("Failed to create team 1");
    repo.create(&make_team(&user.id.as_str(), "Team Beta"))
        .await
        .expect("Failed to create team 2");

    let teams = repo
        .list_by_owner(&user.id.as_str())
        .await
        .expect("Failed to list teams");
    assert!(teams.len() >= 2);

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_update_team() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TeamRepository::new(pool.clone());

    let team = make_team(&user.id.as_str(), "Before Update");
    let created = repo.create(&team).await.expect("Failed to create team");

    let mut updated_team = created.clone();
    updated_team.name = "After Update".to_string();
    updated_team.description = Some("Updated description".to_string());
    updated_team.updated_at = chrono::Utc::now();

    let updated = repo
        .update(&updated_team)
        .await
        .expect("Failed to update team");
    assert_eq!(updated.name, "After Update");
    assert_eq!(updated.description.as_deref(), Some("Updated description"));

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_delete_team() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let user = create_test_user(&pool).await;
    let repo = TeamRepository::new(pool.clone());

    let team = make_team(&user.id.as_str(), "To Delete");
    let created = repo.create(&team).await.expect("Failed to create team");

    repo.delete(&created.id)
        .await
        .expect("Failed to delete team");

    let result = repo.get_by_id(&created.id).await;
    assert!(result.is_err(), "Deleted team should not be found");

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_add_and_list_team_members() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let owner = create_test_user(&pool).await;
    let member_user = create_test_user(&pool).await;
    let repo = TeamRepository::new(pool.clone());

    let team = make_team(&owner.id.as_str(), "Member Team");
    let created = repo.create(&team).await.expect("Failed to create team");

    let team_member = tachyon_database::TeamMember::new(
        created.id.clone(),
        member_user.id.as_str(),
        2,
        "editor".to_string(),
    );

    repo.add_member(&team_member)
        .await
        .expect("Failed to add team member");

    let members = repo
        .list_members(&created.id)
        .await
        .expect("Failed to list team members");
    assert!(!members.is_empty());

    teardown_database(&pool).await;
}

#[tokio::test]
async fn test_remove_team_member() {
    if skip_without_db() {
        println!("Skipping: DATABASE_URL not set");
        return;
    }

    let pool = create_test_pool().await;
    setup_database(&pool).await;
    let _ = teardown_database(&pool).await;

    let owner = create_test_user(&pool).await;
    let member_user = create_test_user(&pool).await;
    let repo = TeamRepository::new(pool.clone());

    let team = make_team(&owner.id.as_str(), "Remove Member Team");
    let created = repo.create(&team).await.expect("Failed to create team");

    let team_member = tachyon_database::TeamMember::new(
        created.id.clone(),
        member_user.id.as_str(),
        2,
        "editor".to_string(),
    );
    repo.add_member(&team_member)
        .await
        .expect("Failed to add team member");

    repo.remove_member(&created.id, &member_user.id.as_str())
        .await
        .expect("Failed to remove team member");

    let is_member = repo
        .is_member(&created.id, &member_user.id.as_str())
        .await
        .expect("Failed to check membership");
    assert!(!is_member);

    teardown_database(&pool).await;
}
