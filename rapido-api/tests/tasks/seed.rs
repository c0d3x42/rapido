use loco_rs::{prelude::*, boot::run_task};
use rapido_api::app::App;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_can_seed_data() {
    let boot = boot_test::<App>().await.unwrap();

    assert!(run_task::<App>(
        &boot.app_context,
        Some(&"seed_data".to_string()),
        &task::Vars::default()
    )
    .await
    .is_ok());
}
