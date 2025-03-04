// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//# init --addresses Test=0x0 A=0x42 --simulator --custom-validator-account

//# publish
module Test::M1 {
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;
    use sui::transfer;
    use sui::coin::Coin;

    struct Object has key, store {
        id: UID,
        value: u64,
    }

    fun foo<T: key, T2: drop>(_p1: u64, value1: T, _value2: &Coin<T2>, _p2: u64): T {
        value1
    }

    public entry fun create(value: u64, recipient: address, ctx: &mut TxContext) {
        transfer::public_transfer(
            Object { id: object::new(ctx), value },
            recipient
        )
    }
}

//# run Test::M1::create --args 0 @A

//# run Test::M1::create --args 0 @validator_0

//# view-object 0,0

//# view-object 2,0

//# view-object 3,0

//# create-checkpoint 4

//# view-checkpoint


//# advance-epoch 6

//# view-checkpoint

//# run-graphql

{
  checkpoint {
    sequenceNumber
  }
}
//# create-checkpoint

//# view-checkpoint

//# run-graphql

{
  checkpoint {
    sequenceNumber
  }
}

//# run-graphql --show-usage --show-headers --show-service-version

{
  checkpoint {
    sequenceNumber
  }
}

//# view-checkpoint

//# advance-epoch

// Demonstrates using variables
// If the variable ends in _opt, this is the optional variant

//# run-graphql --variables A
{
  address(address: $A) {
    objectConnection{
      edges {
        node {
          address
          digest
          kind
        }
      }
    }
  }
}

//# run-graphql --variables Test A obj_2_0 validator_0
{
  address(address: $Test) {
    objectConnection{
      edges {
        node {
          address
          digest
          kind
        }
      }
    }
  }
  second: address(address: $A) {
    objectConnection{
      edges {
        node {
          address
          digest
          kind
        }
      }
    }
  }

  val_objs: address(address: $validator_0) {
    objectConnection{
      edges {
        node {
          address
          digest
          kind
            owner {
            address
          }
        }
      }
    }
  }

  object(address: $obj_2_0) {
    version
    owner {
      address
    }
  }

}


//# view-graphql-variables
// List all the graphql variables


//# run-graphql --variables validator_0
{
  latestSuiSystemState {
    validatorSet {
      activeValidators {
        address {
          address
        }
      }
    }
  }
  address(address: $validator_0) {
    address
  }
}
