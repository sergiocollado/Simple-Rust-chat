reference: http://www.csc.villanova.edu/~mprobson/courses/sp21-csc2405/chat.html Chat Client and Multi-Threaded Chat Server

## Introduction

The goal of this activity is to implement a simplified chat server that can support multiple clients over the Internet. There are a seemingly infinite number of chat programs out there using various protocols, with IRC (Internet Relay Chat) being one of the earliest and most popular chat protocols. We will implement a very simplified version of IRC.

## Complete the Chat Client/Server Application

Extend the code in the chat client and server template to implement a chat application. Users should be able to join the chat server after entering their names, broadcast messages to all other users, and leave the chat room anytime.

Specifically, the client and server should implement the following communication protocol (the client reads in commands from the user and forwards them to the server):


### JOIN name (Example: JOIN Melissa)
The chat client forwards the request to join to the server. When the server receives this request from the client, it adds that client to a list of clients involved in the chat session.


### LEAVE
The chat client forwards the request to leave to the server. When the server receives the request to leave the chat session from the client, it removes that client from its list of clients involved in the chat session.
The client should not be able to invoke the LEAVE command before joining the chat session.


### WHO
The chat client forwards this request to the server. The server responds back with a list of names of those who have joined the chat session, one per line. Once the client receives this list, it displays it on the screen.


### HELP
The client prints out a list of available commands.

