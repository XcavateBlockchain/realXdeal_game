use crate::{mock::*, Error, Event};
use frame_support::{
	assert_noop, assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use sp_runtime::traits::BadOrigin;

fn current_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 0 {
			GameModule::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::reset_events();
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		GameModule::on_initialize(System::block_number());
	}
}

#[test]
fn setup_game_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
	});
}

#[test]
fn setup_game_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(GameModule::setup_game(RuntimeOrigin::signed([0; 32].into())), BadOrigin);
	});
}

#[test]
fn play_game_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::give_points(RuntimeOrigin::root(), [0; 32].into()));
		current_block(222);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player, 
			245
		));
		assert_eq!(GameModule::game_info(0).unwrap().property.id, 2);
	});
}
