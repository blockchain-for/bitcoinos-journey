use core::time;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use colored::Colorize;

use crate::{
    block::{Block, ProposedBlock},
    crypto,
    node::{Node, DIFFICULTY},
};

pub fn start_miner(node: Arc<Mutex<Node>>, interrupt_tx: mpsc::Receiver<()>) {
    let (out_tx, out_rx) = mpsc::channel();
    let (in_tx, in_rx) = mpsc::channel();

    thread::spawn(move || loop {
        if let Ok(proposed_block) = out_rx.try_recv() {
            let proposed_block: ProposedBlock = proposed_block;
            let mut nonce = 0u32;
            let block_string = proposed_block.serialize();

            loop {
                if let Ok(()) = interrupt_tx.try_recv() {
                    in_tx.send(None).unwrap();
                    break;
                }

                let block = format!("{}{}", block_string, nonce);
                let block_hash = hex::encode(crypto::sha256(block.clone()));

                if block_hash.starts_with(&"0".repeat(DIFFICULTY)) {
                    let mined_block = Block {
                        hash: block_hash,
                        nonce,
                        prev_block: proposed_block.prev_block,
                        transactions: proposed_block.transactions,
                    };

                    in_tx.send(Some(mined_block)).unwrap();

                    break;
                }

                thread::sleep(time::Duration::from_millis(100));
                nonce += 1;
            }
        }

        thread::sleep(time::Duration::from_millis(500));
    });

    {
        let mut node = node.lock().unwrap();
        let proposed_block = node.get_proposed_block().unwrap();
        out_tx.send(proposed_block).unwrap();
    }

    loop {
        if let Ok(block) = in_rx.try_recv() {
            let mut node = node.lock().unwrap();
            block.iter().for_each(|b| {
                println!("{} {}", "Minted block:".green(), b.hash);
                node.receive_block(b).unwrap();
            });

            let proposed_block = node.get_proposed_block().unwrap();
            out_tx.send(proposed_block).unwrap();
        }

        thread::sleep(time::Duration::from_millis(1000));
    }
}
