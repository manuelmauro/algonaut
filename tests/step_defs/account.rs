use crate::step_defs::world::World;
use cucumber::then;

#[then(
    "I get the account address for the current application and see that it matches the app id's hash"
)]
async fn assert_app_account_is_the_hash(w: &mut World) {
    let _app_id = w.app_id;
    // TODO
}
