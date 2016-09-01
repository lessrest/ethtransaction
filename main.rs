// Copyright 2016 Nexus Development

// ethtransaction is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// ethtransaction is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with ethtransaction.  If not, see <http://www.gnu.org/licenses/>.

//! Generate Ethereum transactions

extern crate docopt;
extern crate rustc_serialize;

extern crate ethcore;
extern crate ethcore_util as util;

use ethcore::transaction::{Transaction, Action};
use rustc_serialize::hex::FromHexError;
use std::io::Write;
use std::str::FromStr;
use util::{H160, U256, FromHex, Uint, Stream, FromDecStrErr, ToPretty};

const USAGE: &'static str = r#"
Generate an Ethereum transaction

Usage:
  ethtransaction -h
  ethtransaction hex [options] <calldata>
  ethtransaction bin [options] <calldata>

Options:
  --to=<account>          Transaction recipient (omit for contract creation)
  --nonce=<nonce>         Nonce of sender
  --value=<value>         Value to send [default: 0]
  --gas=<gas>             Gas
  --gasprice=<limit>      Gas price
  -h --help               Show this screen
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_to: Option<String>,
  flag_nonce: String,
  flag_value: Option<String>,
  flag_gas: String,
  flag_gasprice: String,
  arg_calldata: String,
  cmd_hex: bool,
  cmd_bin: bool,
}

impl Args {
  pub fn action(&self) -> Result<Action, FromHexError> {
    match self.flag_to {
      Some(ref s) => Ok(Action::Call(try!(H160::from_str(s)))),
      _ => Ok(Action::Create)
    }
  }

  pub fn value(&self) -> Result<U256, FromDecStrErr> {
    match self.flag_value {
      Some(ref s) => Ok(try!(U256::from_dec_str(s))),
      _ => Ok(U256::from(0))
    }
  }
}

fn run(args: &Args) -> Result<Vec<u8>, String> {
  let t = Transaction {
    action: try!(
      args.action().or(Err("invalid --to"))
    ),
    value: try!(
      args.value().or(Err("invalid --value"))
    ),
    gas: try!(
      U256::from_dec_str(&args.flag_gas).or(Err("invalid --gas"))
    ),
    gas_price: try!(
      U256::from_dec_str(&args.flag_gasprice).or(Err("invalid --gasprice"))
    ),
    nonce: try!(
      U256::from_dec_str(&args.flag_nonce).or(Err("invalid --nonce"))
    ),
    data: try!(
      args.arg_calldata.from_hex().or(Err("invalid calldata"))
    )
  };

  let mut stream = util::rlp::RlpStream::new();
  t.rlp_append_unsigned_transaction(&mut stream);

  Ok(stream.drain().to_vec())
}

fn main() {
  let args: Args = docopt::Docopt::new(USAGE).and_then(|d| d.decode())
    .unwrap_or_else(|_e| {
      println!("{}", USAGE);
      std::process::exit(1);
    });

  match run(&args) {
    Ok(vec) => {
      if args.cmd_hex {
        println!("{}", vec.to_hex());
      } else {
        std::io::stdout().write(&vec).expect("write failed");
      }
    },
    Err(e) => {
      println!("{}", e);
      std::process::exit(1)
    }
  }
}