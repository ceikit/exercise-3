#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
    debug, decl_module, decl_storage, decl_event, decl_error, StorageValue, StorageDoubleMap,
    traits::Randomness, RuntimeDebug,
};
use sp_io::hashing::blake2_128;
use frame_system::{ensure_signed, Account};

// #[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
// pub struct Kitty(pub [u8; 16]);

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Kitty(pub [u8; 16], pub Gender);


pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
	trait Store for Module<T: Config> as Kitties {
		/// Stores all the kitties, key is the kitty id
		pub Kitties get(fn kitties): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) u32 => Option<Kitty>;
		/// Stores the next kitty ID
		pub NextKittyId get(fn next_kitty_id): u32;
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
	{
		/// A kitty is created. \[owner, kitty_id, kitty\]
		KittyCreated(AccountId, u32, Kitty),
		/// Cannot create Kitty. \[owner, next_kitty_id\]
		CannotCreate(AccountId, u32),
		/// A kitty is created. \[owner, kitty_id, kitty\]
		KittyBred(AccountId, u32, Kitty),
		/// Cannot create Kitty. \[owner, next_kitty_id\]
		CannotBreed(AccountId, u32),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		KittiesIdOverflow,
		SameSexKitties,
		InvalidKittyId
	}
}

fn find_gender(sex: u8) -> Gender {
    if sex % 2 == 0 { Gender::Male } else { Gender::Female }
}


fn generate_dna<T: Config>(sender: &T::AccountId) -> [u8; 16] {
    // Generate a random 128bit value
    let payload = (
        <pallet_randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
        &sender,
        <frame_system::Module<T>>::extrinsic_index(),
    );
    return payload.using_encoded(blake2_128);
}


decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Breed
		#[weight = 1000]
		pub fn breed(origin, kitty_id_1: u32, kitty_id_2: u32) {
			let sender = ensure_signed(origin)?;
			let kitty1 = Self::kitties(&sender, kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty2 = Self::kitties(&sender, kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			// check sex is different
			if kitty1.1 == kitty2.1 {return Err(Error::<T>::SameSexKitties.into());}

			let kitty_id = Self::next_kitty_id();

			let a = kitty1.0;
			let b = kitty2.0;

			// generate dna for bred kitty
			let mut dna3: [u8; 16] = [0; 16];

			for i in 0..16 {
				dna3[i] = if i%2==0 {a[i]} else {b[i]}
			}

			let gender = find_gender(dna3[0]) ;

			// Create and store kitty
			let newKitty = Kitty (dna3, gender);
			debug::info!("BRED KITTY: {:?}", newKitty);

			Kitties::<T>::insert(&sender, kitty_id, &newKitty);
			NextKittyId::put(kitty_id + 1);
			let breed_event = RawEvent::KittyBred(sender, kitty_id, newKitty.clone());

			debug::info!("BREED EVENT: {:?} | next_id: {:?}", breed_event, Self::next_kitty_id());
			Self::deposit_event(breed_event);
		}

		/// Create a new kitty
		#[weight = 1000]
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;

			// TODO: ensure kitty id does not overflow
			let kitty_id = Self::next_kitty_id();
			let kitty_found: Option<Kitty> = Kitties::<T>::get(sender.clone(), kitty_id);

			let prova: Option<Kitty> = Kitties::<T>::get(sender.clone(), 0);
			debug::info!("PROVA: {:?}", prova);
			debug::info!("next id: {:?} | kitty_found: {:?}", kitty_id, kitty_found);

			if kitty_found.is_some()
			{
				Self::deposit_event(RawEvent::CannotCreate(sender.clone(), kitty_id));
			 	return Err(Error::<T>::KittiesIdOverflow.into());
			}


			let dna = generate_dna::<T>(&sender);
			let gender = find_gender(dna[0]) ;

			// Create and store kitty
			// let kitty = Kitty(dna, kitty_id);
			let kitty = Kitty (dna, gender);
			debug::info!("NEW KITTY: {:?}", kitty);

			Kitties::<T>::insert(&sender, kitty_id, &kitty);
			NextKittyId::put(kitty_id + 1);

			// Emit event
			let creation_event = RawEvent::KittyCreated(sender, kitty_id, kitty.clone());
			debug::info!("CREATION EVENT: {:?} | next_id: {:?}", creation_event, Self::next_kitty_id());

			Self::deposit_event(creation_event);
			debug::info!("EVENT CREATION DEPOSITED");
		}
	}
}
