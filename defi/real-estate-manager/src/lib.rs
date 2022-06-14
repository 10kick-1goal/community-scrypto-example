//! [RealEstate] is the blueprint for autorities to manage real estate rights by NFT, also a real estate market place for citizen.
//! In other words, people can buy, sell real estate rights through this blueprint.
//! This blueprint also contain a taxing mechanism for any traded real estate.

use scrypto::prelude::*;

/// The NFTs of real estates, can contain both land right's NFT and building right's NFT (if that land contain a building)
#[derive(TypeId, Encode, Decode, Describe)]
pub enum RealEstate {
    Land(Bucket),
    LandandBuilding(Bucket, Bucket),
}

/// The data of real estates, can contain both land data and building data (if that land contain a building)
#[derive(TypeId, Encode, Decode, Describe)]
pub enum RealEstateData {
    Land(Decimal, String),
    LandandBuilding(Decimal, String, Decimal, u32),
}

/// The land divide construction structure with needed input
#[derive(TypeId, Encode, Decode, Describe)]
pub struct Divide {
    real_estate: RealEstateData, 
    real_estate1_data: RealEstateData, 
    real_estate2_data: RealEstateData, 
    building_on_land: String,
}

/// The land merge construction structure with needed input
#[derive(TypeId, Encode, Decode, Describe)]
pub struct Merge {
    real_estate: RealEstateData, 
    land_right2: Bucket, 
    is_next_to: bool
}

/// The new building construction structure with needed input
#[derive(TypeId, Encode, Decode, Describe)]
pub struct ConstructBuilding {
    real_estate: RealEstateData, 
    building_size: Decimal, 
    building_floor: u32
}

/// The existed building re-construction structure with needed input
#[derive(TypeId, Encode, Decode, Describe)]
pub struct ReConstructBuilding {
    real_estate: RealEstateData, 
    building_size: Decimal, 
    building_floor: u32
}

/// The type of construction
#[derive(TypeId, Encode, Decode, Describe)]
pub enum ConstructionType {
    Divide(Divide),
    Merge(Merge),
    ConstructBuilding(ConstructBuilding),
    ReConstructBuilding(ReConstructBuilding)
}

/// The building right's NFT
/// All the building data (size, floor) is mutable through construction.
/// size (m2)
#[derive(NonFungibleData, TypeId, Encode, Decode, Describe, Copy, Clone)]
pub struct Building {
    #[scrypto(mutable)]
    size: Decimal,
    #[scrypto(mutable)]
    floor: u32,
}

/// The land right's NFT
/// The land data (size, location) is immutable, but the building that land contain is mutable through construction.
/// One land can only contain one building. If people want to contain more, they need to divide the land first. 
/// Also, a the existed real estate with building can only merge with a land with no building.
/// This approach is more asset oriented when managing 2 types of asset that's linked to each other.
/// One individual, organization, company,... can have many real estate and group them with each other however they want.
/// size (m2)
#[derive(NonFungibleData)]
pub struct Land {
    size: Decimal,
    location: String,
    #[scrypto(mutable)]
    contain: Option<(NonFungibleId, Building)>
}

/// The NFT keep track of new construction badge. This can be authorized badge to divide a land, merge lands, constructe a building, re-constructe a building,...
/// Making change to any real estate NFTs need a construction NFT badge, show that the change can happend or has happened.
/// Authorities, organizations in charge of providing construction badge also need to make sure the change is feasible and won't cause bad effect to society.
#[derive(NonFungibleData)]
pub struct Construction {
    construction_type: ConstructionType
}

/// The NFT keep track of real estate seller's order
#[derive(NonFungibleData)]
pub struct Order {
    order_id: NonFungibleId
}

blueprint! {
    struct RealEstateManager {
        /// Component controller badge
        controller_badge: Vault,
        /// Building address
        building: ResourceAddress,
        /// Land address
        land: ResourceAddress,
        /// Tax (%)
        tax: Decimal,
        /// The medium token using for payment 
        token: ResourceAddress,
        /// Badge to track orders on the real estate market
        order_badge: ResourceAddress,
        /// The order book of real estate market
        book: HashMap<NonFungibleId, (Decimal, Option<NonFungibleId>, bool)>,
        /// The Vault contain real estate on sale
        order_vault: Vault,
        /// The Vault contain building on sale with the attached real estate
        order_contain_building: Vault,
        /// Buyer payment vault
        payment_vault: Vault,
        /// Authority's tax vault
        tax_vault: Vault,
        /// Construction authority badge resource address, provide for authorities, organizations in charge of providing construction badge NFT.
        construction_authority_badge: ResourceAddress,
        /// Construction badge NFT resource address.
        construction_badge: ResourceAddress

    }

    impl RealEstateManager {
        
        /// This function will create new Real Estate Manager component
        /// Input: tax percent and the medium token used for payment on market
        /// Output: Component address and the authority badge
        pub fn new(tax: Decimal, medium_token: ResourceAddress) -> (ComponentAddress, Bucket) {
           
            let controller_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Mint Badge")
                .initial_supply(dec!(1));

            let land = ResourceBuilder::new_non_fungible()
                .metadata("name", "Land")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let building = ResourceBuilder::new_non_fungible()
                .metadata("name", "Building")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let real_estate_authority = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Authority Badge")
                .initial_supply(dec!(1));

            let order_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Position Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let construction_authority_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Authority Badge")
                .mintable(rule!(require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(controller_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let construction_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Position Badge")
                .mintable(rule!(require(construction_authority_badge) || require(controller_badge.resource_address())), LOCKED)
                .burnable(rule!(require(construction_authority_badge) || require(controller_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(construction_authority_badge)), LOCKED)
                .no_initial_supply();

            let rules = AccessRules::new()
                .method("new_land", rule!(require(real_estate_authority.resource_address())))
                .method("divide_land", rule!(require(real_estate_authority.resource_address())))
                .method("merge_land", rule!(require(real_estate_authority.resource_address())))
                .method("construct_new_building", rule!(require(real_estate_authority.resource_address())))
                .method("reconstruct_a_building", rule!(require(real_estate_authority.resource_address())))
                .method("take_tax", rule!(require(real_estate_authority.resource_address())))
                .default(rule!(allow_all));

            let comp = Self {

                controller_badge: Vault::with_bucket(controller_badge),
                building: building,
                land: land,
                tax: tax/dec!(100),
                token: medium_token,
                order_badge: order_badge,
                book: HashMap::new(),
                order_vault: Vault::new(land),
                order_contain_building: Vault::new(building),
                payment_vault: Vault::new(medium_token),
                tax_vault: Vault::new(medium_token),
                construction_authority_badge: construction_authority_badge,
                construction_badge: construction_badge
                
            }
            .instantiate()
            .add_access_check(rules)
            .globalize();

            return (comp, real_estate_authority)
        }

        /// This method is for authority to create and distribute new real estate right's NFTs with the input data
        /// Input: real estate data:
        /// - If the real estate have no building > input Enum("Land", Decimal("${land_size}"), "${location}");
        /// - If the real estate contain a building > input Enum("LandandBuilding", Decimal("${land_size}"), "${location}", Decimal("${building_size}"), ${building_floor}u32);
        /// is_overlap: data from Oracle > see if the land is overlap with an existed real estate or not
        /// Output: 
        /// - If the real estate have no building > The land right's NFT;
        /// - If the real estate contain a building > The land right's NFT and the building right's NFT.
        pub fn new_land(&self, data: RealEstateData, is_overlap: bool) -> RealEstate {

            assert!(!is_overlap,
                "This location is overlap with existed real estate."
            );

            match data {

                RealEstateData::Land(land_size, location) => {

                    assert!(land_size > dec!(0),
                        "Must provide the right land size"
                    );

                    let new_land = Land {
                        contain: None,
                        size: land_size,
                        location: location.clone()
                    };

                    let land_id: NonFungibleId = NonFungibleId::random();

                    let land_right = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id, new_land)
                    });

                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_size, location.clone());

                    RealEstate::Land(land_right)

                }

                RealEstateData::LandandBuilding(land_size, location, building_size, building_floor) => {

                    assert!(land_size > dec!(0),
                        "Must provide the right land size."
                    );
        
                    assert!(
                        (building_size > dec!(0)) & (building_floor > 0),
                        "Must provide the right building info."
                    );
        
                    let new_building = Building {
                        size: building_size,
                        floor: building_floor
                    };
        
                    let building_id: NonFungibleId = NonFungibleId::random();
        
                    let building_right = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.building)
                            .mint_non_fungible(&building_id, new_building.clone())
                    });
        
                    let new_land = Land {
                        contain: Some((building_id, new_building)),
                        size: land_size,
                        location: location.clone()
                    };
        
                    let land_id: NonFungibleId = NonFungibleId::random();
        
                    let land_right = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id, new_land)
                    });

                    info!("You have created a new land right's NFT of the {}m2 land on {} with a building right's NFT of the {}m2, {} floor building", land_size, location.clone(), building_size, building_floor);
        
                    RealEstate::LandandBuilding(land_right, building_right)
                }
            }
        }

        pub fn get_real_estate_data(&self, real_estate: RealEstate) -> (RealEstate, RealEstateData) {

            match real_estate {

                RealEstate::Land(land_right) => {

                    assert!(land_right.resource_address()==self.land,
                        "Wrong resource."
                    );
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.is_none(),
                        "This land contain a building, you should also input the building right's NFT."
                    );

                    return (RealEstate::Land(land_right), RealEstateData::Land(land_data.size, land_data.location))

                }

                RealEstate::LandandBuilding(land_right, building_right) => {

                    assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                        "Wrong resource."
                    );
        
                    let building_id = building_right.non_fungible::<Building>().id();

                    let building_data: Building = building_right.non_fungible().data();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.unwrap().0 == building_id,
                        "This land doesn't contain the building from provided building right."
                    );

                    return (RealEstate::LandandBuilding(land_right, building_right), RealEstateData::LandandBuilding(land_data.size, land_data.location, building_data.size, building_data.floor))

                }
            }
        }

        pub fn check_construction_data(&self, construction_data: ConstructionType) -> ConstructionType {

            return construction_data
            
        }

        pub fn new_construction_badge(&self, construction_data: ConstructionType) -> Bucket {

            let checked_construction_data = self.check_construction_data(construction_data);

            let construction = Construction {
                construction_type: checked_construction_data
            };

            let construction_id = NonFungibleId::random();

            let construction_right = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.construction_badge)
                    .mint_non_fungible(&construction_id, construction)
            });

            return construction_right

        }

        /// This method is for authority to divide an existed real estate to 2 other real estates with attached NFTs.
        /// Input: 
        /// real_estate:
        /// - If the real estate have no building > input the land right's NFT
        /// - If the real estate contain a building > input both the land right's NFT and the building right's NFT
        /// real_estate1_data, real_estate2_data: input Enum("Land", Decimal("${land_size}"), "${location}"), these data can also be acquired through Oracle.
        /// building_on_land: data from an Oracle > see the location of building (if the original real estate contain a building) belong to the real estate 1 or real estate 2.
        /// this data is either Default if the original real estate have no building, "1" if the building is on real estate 1, "2" if the building is on real estate 2.
        /// To follow the asset oriented logic, the divided real estate should not have a building which the original real estate didn't have.
        /// Output: 2 divided real estate with the attached NFTs.
        pub fn divide_land(&self, real_estate: RealEstate, real_estate1_data: RealEstateData, real_estate2_data: RealEstateData, building_on_land: String) -> (RealEstate, RealEstate) {

            match real_estate {

                RealEstate::Land(land_right) => {

                    assert!(land_right.resource_address()==self.land,
                        "Wrong resource."
                    );
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.is_none(),
                        "This land contain a building, you should also input the building right's NFT."
                    );
        
                    let result = match real_estate1_data {

                        RealEstateData::Land(land_size1, location1) => {
                            
                            match real_estate2_data {
                                
                                RealEstateData::Land(land_size2, location2) => {

                                    assert!((land_size1+land_size2==land_data.size) & (land_size1>dec!(0)) & (land_size2>dec!(0)),
                                        "Wrong land size data provided!"
                                    );

                                    let new_land1 = Land {
                                        contain: None,
                                        size: land_size1,
                                        location: location1.clone()
                                    };
                
                                    let land_id1: NonFungibleId = NonFungibleId::random();
                
                                    let land_right1 = self.controller_badge.authorize(|| {
                                        borrow_resource_manager!(self.land)
                                            .mint_non_fungible(&land_id1, new_land1)
                                    });
                
                                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_size1, location1.clone());

                                    let new_land2 = Land {
                                        contain: None,
                                        size: land_size2,
                                        location: location2.clone()
                                    };
                
                                    let land_id2: NonFungibleId = NonFungibleId::random();
                
                                    let land_right2 = self.controller_badge.authorize(|| {
                                        borrow_resource_manager!(self.land)
                                            .mint_non_fungible(&land_id2, new_land2)
                                    });
                
                                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_size2, location2.clone());

                                    info!("You have divided the {} land into a {}m2 land on {} and a {}m2 land on {}", land_data.location, land_size1, location1.clone(), land_size2, location2.clone());

                                    (RealEstate::Land(land_right1), RealEstate::Land(land_right2))
                                }
                                RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                            }
                        }
                        RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                    };

                    self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .burn(land_right)
                    });

                    return result

                }

                RealEstate::LandandBuilding(land_right, building_right) => {

                    assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                        "Wrong resource."
                    );
        
                    let building_id = building_right.non_fungible::<Building>().id();

                    let building_data: Building = building_right.non_fungible().data();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.unwrap().0 == building_id,
                        "This land doesn't contain the building from provided building right."
                    );

                    let result = match real_estate1_data {

                        RealEstateData::Land(land_size1, location1) => {
                            
                            match real_estate2_data {
                                
                                RealEstateData::Land(land_size2, location2) => {

                                    assert!((land_size1+land_size2==land_data.size) & (land_size1>dec!(0)) & (land_size2>dec!(0)),
                                        "Wrong land size data provided!"
                                    );

                                    let result2 = if building_on_land == "1" {

                                        let new_land1 = Land {
                                            contain: Some((building_id, building_data)),
                                            size: land_size1,
                                            location: location1.clone()
                                        };
                    
                                        let land_id1: NonFungibleId = NonFungibleId::random();
                    
                                        let land_right1 = self.controller_badge.authorize(|| {
                                            borrow_resource_manager!(self.land)
                                                .mint_non_fungible(&land_id1, new_land1)
                                        });

                                        info!("You have created a new land right's NFT of the {}m2 land on {} with an attached building", land_size1, location1.clone());

                                        let new_land2 = Land {
                                            contain: None,
                                            size: land_size2,
                                            location: location2.clone()
                                        };
                    
                                        let land_id2: NonFungibleId = NonFungibleId::random();
                    
                                        let land_right2 = self.controller_badge.authorize(|| {
                                            borrow_resource_manager!(self.land)
                                                .mint_non_fungible(&land_id2, new_land2)
                                        });
                    
                                        info!("You have created a new land right's NFT of the {}m2 land on {}", land_size2, location2.clone());

                                        info!("You have divided the {} land into a {}m2 land on {} with an attached building and a {}m2 land on {}", land_data.location, land_size1, location1.clone(), land_size2, location2.clone());

                                        (RealEstate::LandandBuilding(land_right1, building_right), RealEstate::Land(land_right2))

                                    } else if building_on_land == "2" {

                                        let new_land1 = Land {
                                            contain: None,
                                            size: land_size1,
                                            location: location1.clone()
                                        };
                    
                                        let land_id1: NonFungibleId = NonFungibleId::random();
                    
                                        let land_right1 = self.controller_badge.authorize(|| {
                                            borrow_resource_manager!(self.land)
                                                .mint_non_fungible(&land_id1, new_land1)
                                        });

                                        info!("You have created a new land right's NFT of the {}m2 land on {}", land_size1, location1.clone());

                                        let new_land2 = Land {
                                            contain: Some((building_id, building_data)),
                                            size: land_size2,
                                            location: location2.clone()
                                        };
                    
                                        let land_id2: NonFungibleId = NonFungibleId::random();
                    
                                        let land_right2 = self.controller_badge.authorize(|| {
                                            borrow_resource_manager!(self.land)
                                                .mint_non_fungible(&land_id2, new_land2)
                                        });
                    
                                        info!("You have created a new land right's NFT of the {}m2 land on {} with an attached building", land_size2, location2.clone());

                                        info!("You have divided the {} land into a {}m2 land on {} and a {}m2 land on {} with an attached building", land_data.location, land_size1, location1.clone(), land_size2, location2.clone());

                                        (RealEstate::Land(land_right1), RealEstate::LandandBuilding(land_right2, building_right))

                                    } else {
                                        panic!("Wrong Oracle data!")
                                    };

                                    result2

                                }
                                RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                            }
                        }
                        RealEstateData::LandandBuilding(_,_,_,_) => { panic!("This land data shoudn't contain a building!")}
                    };
        
                    self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .burn(land_right)
                    });

                    return result

                }
            }    
        }

        /// This method is for authority to merge an existed real estate with an existed (non-building) land.
        /// Input: 
        /// real_estate:
        /// - If the real estate have no building > input the land right's NFT.
        /// - If the real estate contain a building > input both the land right's NFT and the building right's NFT.
        /// land_right2: input the land_right's bucket.
        /// is_next_to: data from an Oracle > see if the 2 land next to each other or not.
        /// To follow the asset oriented logic, the merged land should not have more than 2 buildings.
        /// Output: 1 merged real estate with the attached NFTs. The location data of new real estate is equal to the first original real estate.
        pub fn merge_land(&self, real_estate: RealEstate, land_right2: Bucket, is_next_to: bool) -> RealEstate {

            assert!(land_right2.resource_address() == self.land,
                "Wrong resource"
            );

            assert!(is_next_to,
                "The land should be next to each other for the merge!"
            );

            let land_data2: Land = land_right2.non_fungible().data();

            assert!(land_data2.contain.is_none(),
                        "This land shouldn't contain a building."
            );

            match real_estate {

                RealEstate::Land(land_right) => {

                    assert!(land_right.resource_address()==self.land,
                        "Wrong resource."
                    );
        
                    let land_data: Land = land_right.non_fungible().data();

                    assert!(land_data.contain.is_none(),
                        "This land contain a building, you should also input the building right's NFT."
                    );
        
                    let new_land = Land {
                        contain: None,
                        size: land_data.size+land_data2.size,
                        location: land_data.location.clone()
                    };

                    let land_id_merged: NonFungibleId = NonFungibleId::random();

                    let land_right_merged = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id_merged, new_land)
                    });

                    info!("You have created a new land right's NFT of the {}m2 land on {}", land_data.size+land_data2.size, land_data.location.clone());

                    info!("You have merged the {}m2 land on {} and {}m2 land on {} into a {}m2 land on {}", land_data.size, land_data.location.clone(), land_data2.size, land_data2.location.clone(), land_data.size+land_data2.size, land_data.location.clone());

                    self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .burn(land_right);
                        borrow_resource_manager!(self.land)
                            .burn(land_right2);
                    });

                    return RealEstate::Land(land_right_merged)

                }

                RealEstate::LandandBuilding(land_right, building_right) => {

                    assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                        "Wrong resource."
                    );
        
                    let building_id = building_right.non_fungible::<Building>().id();

                    let building_data = building_right.non_fungible::<Building>().data();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.clone().unwrap().0 == building_id,
                        "This land doesn't contain the building from provided building right."
                    );
        
                    let new_land = Land {
                        contain: Some((building_id, building_data)),
                        size: land_data.size+land_data2.size,
                        location: land_data.location.clone()
                    };

                    let land_id_merged: NonFungibleId = NonFungibleId::random();

                    let land_right_merged = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .mint_non_fungible(&land_id_merged, new_land)
                    });

                    info!("You have created a new land right's NFT of the {}m2 land on {} with an attached building", land_data.size+land_data2.size, land_data.location.clone());

                    info!("You have merged the {}m2 land on {} with an attached building and {}m2 land on {} into a {}m2 land on {} with an attached building", land_data.size, land_data.location.clone(), land_data2.size, land_data2.location.clone(), land_data.size+land_data2.size, land_data.location.clone());

                    self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.land)
                            .burn(land_right);
                        borrow_resource_manager!(self.land)
                            .burn(land_right2);
                    });

                    return RealEstate::LandandBuilding(land_right_merged, building_right)

                }
            }    
        }

        /// This method is for authority to add new building to an existed land.
        /// Input: The land right's NFT proof and building's data.
        /// Output: The building right's NFT of that land.
        pub fn construct_new_building(&self, land_right: Proof, building_size: Decimal, building_floor: u32) -> Bucket {

            assert!(land_right.resource_address()==self.land,
                "Wrong resource."
            );

            let new_building = Building {
                size: building_size,
                floor: building_floor
            };

            let building_id: NonFungibleId = NonFungibleId::random();

            let building_right = self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.building)
                    .mint_non_fungible(&building_id, new_building.clone())
            });

            let mut data: Land = land_right.non_fungible().data();

            assert!(data.contain.is_none(),
                "This land already has a building."
            );

            data.contain = Some((building_id, new_building));

            let location = data.location.clone();

            self.controller_badge
                .authorize(|| land_right.non_fungible().update_data(data));

            info!("You have added a new building right's NFT of the {}m2, {} floor building attached to the land on {} according to construction data", building_size, building_floor, location);

            return building_right

        }

        /// This method is for authority to modify existed building info.
        /// Input: The land right's NFT proof, the building right's NFT proof of that land and the building's new data.
        /// Output: None.
        pub fn reconstruct_a_building(&self, land_right: Proof, building_right: Proof, building_size: Decimal, building_floor: u32) {

            assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                "Wrong resource."
            );

            let mut building_data: Building = building_right.non_fungible().data();

            let building_id = building_right.non_fungible::<Building>().id();

            let mut land_data: Land = land_right.non_fungible().data();

            assert!(land_data.contain.unwrap().0 == building_id,
                "This land doesn't contain the building from provided building right."
            );

            let location = land_data.location.clone();

            building_data.size = building_size;
            building_data.floor = building_floor;
            land_data.contain = Some((building_id, building_data));

            info!("You have modified the building right's NFT data of the building attached to the {} land according to construction data. New building is {}m2, {} floor", location, building_size, building_floor);

            self.controller_badge
                .authorize(|| {
                    land_right.non_fungible().update_data(land_data);
                    building_right.non_fungible().update_data(building_data)
                });

        }

        /// This method is for seller to sell a real estate right's NFTs.
        /// Input: Real estate's right NFTs:
        /// - If the land have no housing > input Enum("Land", Bucket("${land_right}"));
        /// - If the land contain a building > input Enum("LandandBuilding", Bucket("${land_right}"), Bucket("${building_right}"));
        /// Output: The NFT keep track of real estate seller's order
        pub fn new_sell_order(&mut self, real_estate: RealEstate, price: Decimal) -> Bucket {

            match real_estate {

                RealEstate::Land(land_right) => {

                    assert!(land_right.resource_address()==self.land,
                        "Wrong resource."
                    );
        
                    let land_id = land_right.non_fungible::<Land>().id();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.is_none(),
                        "This land contain a building, you should also input the building right's NFT."
                    );
        
                    let new_position = Order {
                        order_id: land_id.clone()
                    };
        
                    let order_badge = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.order_badge)
                            .mint_non_fungible(&NonFungibleId::random(), new_position)
                    });
        
                    self.book.insert(land_id.clone(), (price, None, false));
        
                    self.order_vault.put(land_right);

                    info!("You have created a sell order no.{} on the {} real estate", land_id, land_data.location);
        
                    return order_badge
                }

                RealEstate::LandandBuilding(land_right, building_right) => {

                    assert!((land_right.resource_address()==self.land) & (building_right.resource_address() == self.building),
                        "Wrong resource."
                    );
        
                    let building_id = building_right.non_fungible::<Building>().id();
        
                    let land_data: Land = land_right.non_fungible().data();
        
                    assert!(land_data.contain.unwrap().0 == building_id,
                        "This land doesn't contain the building from provided building right."
                    );
        
                    let land_id = land_right.non_fungible::<Land>().id();
        
                    let new_position = Order {
                        order_id: land_id.clone()
                    };
        
                    let order_badge = self.controller_badge.authorize(|| {
                        borrow_resource_manager!(self.order_badge)
                            .mint_non_fungible(&NonFungibleId::random(), new_position)
                    });
        
                    self.book.insert(land_id.clone(), (price, Some(building_id), false));
        
                    self.order_vault.put(land_right);
        
                    self.order_contain_building.put(building_right);

                    info!("You have created a sell order no.{} on the {} real estate with an attached building", land_id, land_data.location);
        
                    return order_badge

                }
            }    
        }


        /// This method is for buyer to buy a real estate right's NFTs.
        /// Input: The order id and payment (by medium token).
        /// Output: The real estate's NFTs and payment changes.
        pub fn buy(&mut self, order_id: u64, mut payment: Bucket) -> (RealEstate, Bucket) {

            let order_id = NonFungibleId::from_u64(order_id);

            assert!(payment.resource_address()==self.token,
                "Wrong resource."
            );

            let result = self.book.get(&order_id);

            assert!(result.is_some(),
                "The order book doesn't contain this order id"
            );

            let (price, building, status) = result.unwrap().clone();

            assert!(status==false,
                "This real estate is already bought."
            );
        
            let tax = price*self.tax;
        
            assert!(
                payment.amount()>=(price + tax),
                    "Not enough payment"
                );
        
            match building.clone() {
        
                None => {
        
                    self.payment_vault.put(payment.take(price));
                    self.tax_vault.put(payment.take(price*self.tax));
                    self.book.insert(order_id.clone(), (price, building, true));
                    let land_right = self.order_vault.take_non_fungible(&order_id);
                    let land_location = land_right.non_fungible::<Land>().data().location;
                    info!("You have filled the no.{} order and bought the {} real estate", order_id, land_location);
                    return (RealEstate::Land(land_right), payment)
        
                }
        
                Some(building_id) => {
        
                    self.payment_vault.put(payment.take(price));
                    self.tax_vault.put(payment.take(price*self.tax));
                    self.book.insert(order_id.clone(), (price, building, true));
                    let land_right = self.order_vault.take_non_fungible(&order_id);
                    let land_location = land_right.non_fungible::<Land>().data().location;
                    info!("You have filled the no.{} order and bought the {} real estate with the attached building", order_id, land_location);
                    return (RealEstate::LandandBuilding(land_right, self.order_contain_building.take_non_fungible(&building_id)), payment)
        
                }
              
            }
        }

        /// This is method for seller to cancel an order that haven't been bought.
        /// Input: The order NFT badge.
        /// Output: The real estate right's NFTs.
        pub fn cancel_sell_order(&mut self, order_badge: Bucket) -> RealEstate {

            assert!(order_badge.resource_address()==self.order_badge,
                "Wrong resource."
            );

            let order_id = order_badge.non_fungible::<Order>().data().order_id;

            let (_price, building, status) = self.book.get(&order_id).unwrap().clone();

            assert!(status==false,
                "This real estate is already bought."
            );

            self.book.remove(&order_id);

            self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.order_badge)
                    .burn(order_badge)
            });

            let land_right = self.order_vault.take_non_fungible(&order_id);
            let land_location = land_right.non_fungible::<Land>().data().location;

            info!("You have cancel the sell order no.{} on {} real estate", order_id, land_location);

            match building.clone() {

                None => {
                    return RealEstate::Land(land_right)
                }

                Some(building_id) => {
                    return RealEstate::LandandBuilding(land_right, self.order_contain_building.take_non_fungible(&building_id))
                }

            }

        }

        /// This is method for seller to take the payment.
        /// Input: The order NFT badge.
        /// Output: The real estate right's NFTs.
        pub fn take_payment(&mut self, order_badge: Bucket) -> Bucket {

            assert!(
                order_badge.resource_address()==self.order_badge,
                "Wrong resource."
            );

            let order_id = order_badge.non_fungible::<Order>().data().order_id;

            let (price, _building, status) = self.book.get(&order_id).unwrap().clone();

            assert!(status==true,
                "This real estate haven't bought."
            );

            self.controller_badge.authorize(|| {
                borrow_resource_manager!(self.order_badge)
                    .burn(order_badge)
            });

            info!("You have taken the payment on no.{} order", order_id);

            self.payment_vault.take(price)

        }

        pub fn take_tax(&mut self) -> Bucket {
            self.tax_vault.take_all()
        }
    }
}