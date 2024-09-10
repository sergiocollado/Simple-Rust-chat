use crate::aux::*;
use crate::ClientsNameArray;
use crate::ClientsStreamArray;
use crate::MAX_CLIENTS;
use crate::MAX_MESSAGE_SIZE;
use crate::MAX_NAME_LEN;
use crate::VERSION;

use std::error::Error as OtherError;
use std::fmt;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::str;
use std::sync::Arc;

// handle the different implemented commands
pub fn handle_commands(
    input: &[u8],
    index: usize,
    clients_array: &ClientsNameArray,
    stream_array: &ClientsStreamArray,
) -> Result<(), ClientLeavedError> {
    let str_input = str::from_utf8(input).unwrap();

    if check_join_u8(input) {
        handle_join(input, index, clients_array, stream_array);
    } else if check_version(str_input) {
        handle_version(index, clients_array, stream_array);
    } else if check_who(str_input) {
        handle_who(index, clients_array, stream_array);
    } else if check_leave(str_input) {
        handle_leave(index, clients_array, stream_array)?;
    } else {
        broadcast(input, index, clients_array, stream_array);
    }
    Ok(())
}

pub fn check_command(command: &str, input: &str) -> bool {
    // actually it coud be used the function .starts_with()
    // ref: https://doc.rust-lang.org/std/primitive.slice.html#method.starts_with
    let first_word = first_word(input);
    command.trim() == first_word
}

pub fn check_command_u8(command: &str, input: &[u8]) -> bool {
    let first_word_u8 = first_word_u8(input);
    command == std::str::from_utf8(first_word_u8).unwrap()
}

//fn check_join(input: &str) -> bool {
//    check_command("JOIN", input)
//}

pub fn check_join_u8(input: &[u8]) -> bool {
    check_command_u8("JOIN", input)
}

// check if the WHO command was issued
pub fn check_who(input: &str) -> bool {
    check_command("WHO", input)
}

// check if the LEAVE command was issued.
pub fn check_leave(input: &str) -> bool {
    check_command("LEAVE", input)
}

// check if the VERSION command was issued
pub fn check_version(input: &str) -> bool {
    check_command("VERSION", input)
}

// LEAVE command: removes the user from the chat, and close the connection.
pub fn handle_leave(
    index: usize,
    clients_array: &ClientsNameArray,
    stream_array: &ClientsStreamArray,
) -> Result<(), ClientLeavedError> {
    let name_i = get_client_name_at_position_i(index, &clients_array);
    if name_i.is_some() {
        let name = name_i.unwrap();
        let name_str = std::str::from_utf8(&name[..]).unwrap();
        println!("{} has left the chat", &name_str);
        let mut leave_msg: String = String::new();
        leave_msg.push_str(&name_str);
        leave_msg.push_str(" has left the chat");
        broadcast_msg_to_other_names(leave_msg.as_bytes(), index, clients_array, stream_array);
        remove_client_i(index, clients_array, stream_array);
        Err(ClientLeavedError::new(&name_str))
    } else {
        remove_client_i(index, clients_array, stream_array);
        Ok(())
    }
}

// VERSION command: reports the version of the program.
pub fn handle_version(
    index: usize,
    clients_array: &ClientsNameArray,
    clients_streams: &ClientsStreamArray,
) {
    let name_array_clone = Arc::clone(clients_array);
    let name_arrays_mutex = name_array_clone.lock().unwrap();
    let name_arrays: [Option<[u8; MAX_NAME_LEN]>; MAX_CLIENTS] = *name_arrays_mutex;
    if name_arrays[index].is_some() {
        // send version
        println!(
            "{}",
            str::from_utf8(VERSION)
                .unwrap()
                .to_string()
                .trim_matches(char::from(0))
        );
        send_msg_to_ith_client(VERSION, index, &name_array_clone, clients_streams)
    }
}

// WHO command: list registered participans
pub fn handle_who(
    index: usize,
    clients_array: &ClientsNameArray,
    clients_streams: &ClientsStreamArray,
) {
    if is_user_registered(index, clients_array) {
        let name_array_clone = Arc::clone(clients_array);
        let name_arrays_mutex = name_array_clone.lock().unwrap();
        let name_arrays: [Option<[u8; MAX_NAME_LEN]>; MAX_CLIENTS] = *name_arrays_mutex;
        for name in name_arrays {
            if name.is_some() {
                println!(
                    "{}",
                    str::from_utf8(&name.unwrap())
                        .unwrap()
                        .to_string()
                        .trim_matches(char::from(0))
                );
                send_msg_to_ith_client(&name.unwrap(), index, &name_array_clone, clients_streams)
            }
        }
    }
}

pub fn handle_join(
    input: &[u8],
    index: usize,
    clients_array: &ClientsNameArray,
    clients_streams: &ClientsStreamArray,
) {
    if is_user_registered(index, clients_array) == false {
        let (_, name) = first_2_words(std::str::from_utf8(input).unwrap());
        if name.is_some() {
            let name = name.unwrap();

            // TODO: check if the name already exists in the names_array.

            // copy the name to the client's name array
            let mut client_name: [u8; MAX_NAME_LEN] = [0; MAX_NAME_LEN];
            let mut i: usize = 0;
            // copy the bytes into the name
            while i < MAX_NAME_LEN && i < name.as_bytes().len() {
                client_name[i] = name.as_bytes()[i];
                i = i + 1;
            }

            {
                let clients = Arc::clone(clients_array);
                let mut array_clients = clients.lock().unwrap();
                array_clients[index] = Some(client_name);
            }

            println!("{} has joined the chat", name);
            let join_msg = format!(
                "{} has joined the chat",
                str::from_utf8(&client_name).unwrap()
            );
            broadcast_msg_to_other_names(
                join_msg.as_bytes(),
                index,
                clients_array,
                clients_streams,
            );
        }
    } else {
        // TODO: send messsage to user, to tell you cannont join again
        //
        //send_msg_to_ith_client(message, i, &name_array_clone, clients_streams)
    }
}

// handle the chat client. This funtion will run in a independent thread,
// will check the incomming messages, check for the different commads, and
// execute those commands.
pub fn handle_client(
    mut stream: TcpStream,
    index: usize,
    clients_array: &ClientsNameArray,
    stream_array: &ClientsStreamArray,
) -> Result<(), ClientLeavedError> {
    let mut data = [0 as u8; MAX_MESSAGE_SIZE]; // using 512 byte buffer
    loop {
        let size = stream
            .read(&mut data)
            .expect("error when reading the stream");

        // TODO: FIXME: what happens when size is bigger than MAX_MESSAGE_SIZE?

        server_chat_output(&data, index, size, &clients_array);

        handle_commands(&data, index, clients_array, stream_array)?;

        data = [0; MAX_MESSAGE_SIZE]; // clean the buffer for the next iter
    }
}

// send a given message to all the other chat clients except for the
// one who send the message.
pub fn broadcast(
    message: &[u8],
    index: usize,
    clients_array: &ClientsNameArray,
    clients_streams: &ClientsStreamArray,
) {
    if is_user_registered(index, clients_array) {
        for i in 0..MAX_CLIENTS {
            if i != index
                && clients_array.lock().unwrap()[i].is_some()
                && clients_streams.lock().unwrap()[i].is_some()
            {
                let clients_streams = Arc::clone(&clients_streams);
                let mut stream_mutex = clients_streams.lock().unwrap();
                let stream_option = &mut *stream_mutex;
                let mut stream_i = stream_option[i]
                    .as_mut()
                    .unwrap()
                    .try_clone()
                    .expect("failed to clone a stream");

                let name_i = get_client_name_at_position_i(index, &clients_array);
                if name_i.is_some() {
                    let msg = format!(
                        "[{}] {}",
                        str::from_utf8(&name_i.unwrap()[..]).unwrap(),
                        str::from_utf8(&message).unwrap()
                    );
                    stream_i
                        .write_all(msg.as_bytes())
                        .expect("Failed to send data through a stream");
                }
            }
        }
    }
}

// remove client at ith position, and shut down socket connection if needed
pub fn remove_client_i(
    index: usize,
    clients_array: &ClientsNameArray,
    stream_array: &ClientsStreamArray,
) -> () {
    let clients = Arc::clone(clients_array);
    let mut array_clients = clients.lock().unwrap();
    array_clients[index] = None;
    let stream = Arc::clone(stream_array);
    let mut stream_client = stream.lock().unwrap();
    stream_client[index]
        .as_mut()
        .unwrap()
        .shutdown(Shutdown::Both)
        .expect("Unable to shutdown the stream");
    stream_client[index] = None;
}

// send a given message to the ith chat client.
pub fn send_msg_to_ith_client(
    message: &[u8],
    index: usize,
    clients_array: &ClientsNameArray,
    clients_streams: &ClientsStreamArray,
) {
    let clients_streams = Arc::clone(&clients_streams);
    let mut stream_mutex = clients_streams.lock().unwrap();
    let stream_option = &mut *stream_mutex;
    if stream_option[index].is_some() {
        let mut stream_i = stream_option[index]
            .as_mut()
            .unwrap()
            .try_clone()
            .expect("failed to clone a stream");
        stream_i
            .write_all(message)
            .expect("Failed to send data through a stream");
        stream_i
            .write_all(String::from('\n').as_bytes())
            .expect("Failed endline");
    }
}

// send a message to all the chat clients except for the ith client.
pub fn broadcast_msg_to_other_names(
    message: &[u8],
    client_index: usize,
    clients_array: &ClientsNameArray,
    clients_streams: &ClientsStreamArray,
) {
    let name_array_clone = Arc::clone(clients_array);
    let name_arrays_mutex = name_array_clone.lock().unwrap();
    let name_arrays: [Option<[u8; MAX_NAME_LEN]>; MAX_CLIENTS] = *name_arrays_mutex;
    for (i, &name) in name_arrays.iter().enumerate() {
        if name.is_some() {
            if client_index != i {
                println!(
                    "{}",
                    str::from_utf8(&name.unwrap())
                        .unwrap()
                        .to_string()
                        .trim_matches(char::from(0))
                );
                send_msg_to_ith_client(message, i, &name_array_clone, clients_streams)
            }
        }
    }
}

// retrieve the name of the ith client from the array of names
pub fn get_client_name_at_position_i(
    index: usize,
    clients_array: &ClientsNameArray,
) -> Option<[u8; MAX_NAME_LEN]> {
    let name_array_clone = Arc::clone(&clients_array);
    let name_array_mutex = name_array_clone.lock().unwrap();
    let name_array: [Option<[u8; MAX_NAME_LEN]>; MAX_CLIENTS] = *name_array_mutex;
    let name_i: Option<[u8; MAX_NAME_LEN]> = name_array[index];
    name_i
}

// check if the user at ith position is already registered
pub fn is_user_registered(index: usize, clients_array: &ClientsNameArray) -> bool {
    get_client_name_at_position_i(index, clients_array).is_some()
}

// repeat ith client's message inside the server.
pub fn server_chat_output(
    input: &[u8],
    index: usize,
    size: usize,
    clients_array: &ClientsNameArray,
) {
    let user_name: [u8; MAX_NAME_LEN] = Default::default();
    {
        let name_i = get_client_name_at_position_i(index, &clients_array);
        if name_i.is_some() {
            let name = name_i.unwrap();
            let user_name = name;
            print!(
                "[{}] ",
                std::str::from_utf8(&user_name)
                    .unwrap()
                    .trim_matches(char::from(0))
                // maybe it could be use also: String::from_utf8_lossy()
            );
        }
    }

    std::io::stdout()
        .write_all(&input[0..size])
        .expect("Error writing to stdout");
}

// reference: https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html
#[derive(Debug)]
pub struct ClientLeavedError {
    details: String,
}

impl ClientLeavedError {
    pub fn new(msg: &str) -> ClientLeavedError {
        ClientLeavedError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ClientLeavedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details.trim_matches(char::from(0)))
    }
}

impl OtherError for ClientLeavedError {
    fn description(&self) -> &str {
        &self.details
    }
}
