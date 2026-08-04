# Lab 01 — Regtest network inspection

## Commands used

bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash

## Terminal output

TODO: Record chain, block height, and best-block hash.

## Evidence references
![alt text](image.png)

## Explanation

TODO: 1. Polar

Polar is a tool that helps developers quickly create and manage local Bitcoin and Lightning Network environments. Instead of installing and configuring everything manually, Polar lets you spin up Bitcoin nodes and Lightning nodes with just a few clicks. It's mainly used for learning, testing, and developing Bitcoin and Lightning applications without using real Bitcoin.

// Docker

Docker is a platform that packages an application and everything it needs (such as libraries, dependencies, and configuration) into a container. A container allows the application to run the same way on any computer, regardless of the operating system or environment. This makes it easier to develop, test, and deploy software consistently.

//Bitcoin Core

Bitcoin Core is the official software that implements the Bitcoin protocol. It allows you to run a full Bitcoin node, which downloads and verifies the blockchain, validates transactions and blocks, and communicates with other nodes on the network. Developers also use Bitcoin Core's JSON-RPC interface to build applications that interact with the Bitcoin blockchain, such as creating wallets, sending transactions, and querying blockchain data.

// Regtest

Regtest (Regression Test Mode) is a special private Bitcoin network designed for development and testing. Unlike the public Bitcoin network, regtest lets developers create blocks instantly, generate unlimited test bitcoins, and fully control the blockchain. Because it is isolated from the real network, it provides a safe environment for testing Bitcoin applications without spending real money or waiting for real block confirmations.