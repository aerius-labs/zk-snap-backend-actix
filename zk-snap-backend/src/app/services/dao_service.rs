use std::io::{Error, ErrorKind};

use crate::app::dtos::dao_dto::CreateDaoDto;
use crate::app::entities::dao_entity::Dao;
use crate::app::repository::generic_repository::Repository;
use crate::app::utils::merkle_tree_helper::from_members_to_leaf;
use actix_web::web;
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use mongodb::bson::oid::ObjectId;
use num_bigint::BigUint;
use pse_poseidon::Poseidon;
use voter::merkletree::native::MerkleTree;

pub async fn create_dao(
    db: web::Data<Repository<Dao>>,
    dao: CreateDaoDto,
) -> Result<String, Error> {
    // if dao.members.is_empty() {
    //     return Err(Error::new(ErrorKind::InvalidInput, "Members are required"));
    // }

    // let leaves: Vec<Fr> = from_members_to_leaf(dao.members.as_slice()).unwrap();
    // let mut hash = Poseidon::<Fr, 3, 2>::new(8, 57);
    // let merkle_tree = MerkleTree::new(&mut hash, leaves).unwrap();
    // let root = merkle_tree.get_root().to_bytes();

    // let root = BigUint::from_bytes_le(root.as_slice());

    // let tree = merkle_tree.get_tree();

    let dao_entity = Dao {
        id: Some(ObjectId::new()),
        name: dao.name,
        description: dao.description,
        logo: dao.logo,
        // members: dao.members,
        // members_tree: tree,
        // members_root: root,
    };

    let object_id = match db.create(dao_entity).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };

    Ok(object_id)
}

pub async fn get_all_daos(db: web::Data<Repository<Dao>>) -> Result<Vec<Dao>, Error> {
    match db.find_all().await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

pub async fn get_dao_by_id(db: web::Data<Repository<Dao>>, id: &str) -> Result<Dao, Error> {
    let result = match db.find_by_id(id).await {
        Ok(result) => result,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };
    match result {
        Some(dao) => Ok(dao),
        None => Err(Error::new(ErrorKind::NotFound, "DAO not found")),
    }
}

pub async fn delete_by_id(db: web::Data<Repository<Dao>>, id: &str) -> Result<(), Error> {
    match db.delete(id).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

pub async fn update_dao_by_id(
    db: web::Data<Repository<Dao>>,
    id: &String,
    dao: CreateDaoDto,
) -> Result<(), Error> {
    let obj_id = ObjectId::parse_str(id).unwrap();
    let db_dao = db.find_by_id(id).await.unwrap().unwrap();

    let dao_entity = Dao {
        id: Some(obj_id),
        name: dao.name,
        description: dao.description,
        logo: dao.logo,
        // members: db_dao.members,
        // members_tree: db_dao.members_tree,
        // members_root: db_dao.members_root,
    };

    match db.update(id, dao_entity).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}
