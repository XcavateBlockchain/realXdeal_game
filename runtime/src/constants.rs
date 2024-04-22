/// Money matters.
pub mod currency {
	use node_primitives::Balance;

	pub const MILLICENTS: Balance = 1_000_000_000;
	pub const INIT_SUPPLY_FACTOR: Balance = 100;
	pub const STORAGE_BYTE_FEE: Balance = 20 * MILLICENTS * INIT_SUPPLY_FACTOR;
	pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
	pub const DOLLARS: Balance = 100 * CENTS;

}