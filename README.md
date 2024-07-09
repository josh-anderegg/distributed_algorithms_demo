# Project description
Small demo for testing and creating histories of [Paxos](https://en.wikipedia.org/wiki/Paxos_(computer_science)) runs, both within synchronous and asynchronous networks. The implementation of the Network and the communication system are created in such a way to allow simulating other distributed algorithms. Planned are:
 - Paxos with packet loss and failing nodes
 - King algorithm 
 - [Ben-Or algorithm](https://decentralizedthoughts.github.io/2022-03-30-asynchronous-agreement-part-two-ben-ors-protocol/) for byzantine fault tolerance.