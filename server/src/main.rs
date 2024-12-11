//  Rust languaje references:
// - https://doc.rust-lang.org/book
// - https://doc.rust-lang.org/reference/
// - https://doc.rust-lang.org/rust-by-example
// - https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/
// - https://doc.rust-lang.org/nomicon/
// - https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html
// - https://github.com/rust-lang/rustlings
// - https://github.com/rust-lang/rustlings/tree/rustlings-1
// - little book of rust macros: https://github.com/DanielKeep/tlborm
// - liltle book of rust macros updated: https://github.com/Veykril/tlborm
// - https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html
//
// references for coding the program:
// https://doc.rust-lang.org/std/net/index.html#
// reference: https://doc.rust-lang.org/std/net/struct.TcpListener.html
// reference: https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
// https://github.com/aswathy-Packt/Network-Programming-with-Rust
// https://stevedonovan.github.io/rust-gentle-intro/7-shared-and-networking.html#a-better-way-to-resolve-addresses
// https://stackoverflow.com/questions/63350694/what-are-the-lifetimes-of-while-loop-arguments-using-a-mutex
// https://www.sitepoint.com/rust-global-variables/
// https://profpatsch.de/notes/rust-string-conversions

// generate documentation with: cargo doc --no-deps --open

use std::env;
use std::net::{TcpListener, TcpStream};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;

pub mod aux;
pub mod commands;
use crate::aux::*;
use crate::commands::*;

const MAX_CLIENTS: usize = 20; // max clients cannot be >32, because the way the array initialization is done
const MAX_NAME_LEN: usize = 20;
const MAX_MESSAGE_SIZE: usize = 512;
const MAX_HOSTNAME_SIZE: usize = 50;
const VERSION: &[u8] = b"Simple Rust Chat Server v0.1\n";
// https://www.sitepoint.com/rust-global-variables/
// https://www.howtosolutions.net/2022/12/rust-create-global-variable-mutable-struct-without-unsafe-code-block/

// TODO: create a structure for this
// TODO: preferibly, this may be static
type ClientsStreamArray = Arc<Mutex<[Option<TcpStream>; MAX_CLIENTS]>>;
type ClientsNameArray = Arc<Mutex<[Option<[u8; MAX_NAME_LEN]>; MAX_CLIENTS]>>;

fn verify_arguments(args: &Vec<String>) {
    println!("arguments: {:?}", args);

    if args.is_empty() {
        println!(">>> Incorrect number of arguments. Usage is: ./tcp_server <port number> \ni.e.: ./tcp_server 1153");
        process::exit(1);
    }

    if args.len() > 2 {
        println!(">>> Incorrect number of arguments. Usage is: ./tcp_server <port number> \ni.e.: ./tcp_server 1153");
        process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    verify_arguments(&args);

    let port = &args[1];
    println!("port: {}", port);

    // initialize array of clients
    // ref: https://www.joshmcguigan.com/blog/array-initialization-rust/
    // The primary downside to this method is it only works for arrays up to size 32.
    assert!(MAX_CLIENTS < 32);

    // TODO: create a structure for this
    let clients_streams: ClientsStreamArray = Arc::new(Mutex::new(Default::default()));
    let clients_names: ClientsNameArray = Arc::new(Mutex::new([None; MAX_CLIENTS]));

    // create a listening socket
    let listener =
        TcpListener::bind("0.0.0.0:".to_owned() + port).expect("\nError: Bind failed!\n");

    loop {
        // you could do the same without a loop with `listener.incomming()`.
        match listener.accept() {
            // new connection accepted
            Ok((stream, addr)) => {
                println!("New connection accepted: :{:?}, {:?}", stream, addr);

                for i in 0..MAX_CLIENTS {
                    // check for an empty spot on the clients array
                    //
                    // TODO: try to remove all unwrap(), and use instead
                    // expect(), or unwrap_or(), ? operator, if let ...
                    // or better error handling
                    if clients_streams.lock().unwrap()[i].is_none() {
                        println!("New client: pos({}): {:?}", i, addr);

                        {
                            // include/update this stream, in the array of clientsStreams
                            clients_streams.lock().unwrap()[i] = Some(
                                stream
                                    .try_clone()
                                    .expect("failure trying to clone a stream"),
                            );
                        }

                        let client_names_array = Arc::clone(&clients_names);
                        let client_stream_array = Arc::clone(&clients_streams);

                        thread::spawn(move || {
                            // connection suceeded
                            handle_client(stream, i, &client_names_array, &client_stream_array)
                                .unwrap_or_else(|error| eprintln!("client leaved: {:?}", error));
                        });

                        break; //once the new connection is registered, end the loop.
                    }
                }
            }
            Err(error) => {
                println!("Error: couldn't get client {error:?}");
                break;
            }
        }
    }

    drop(listener); // close the socket server;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn verify_command() {
        assert!(check_command("Hello", "Hello World!"));
    }

    //#[test]
    //fn verify_join() {
    //    assert!(check_join("JOIN Alice"));
    //}

    #[test]
    fn verify_join_u8() {
        assert!(check_join_u8(String::from("  JOIN Alice").as_bytes()));
    }

    #[test]
    fn verify_check_who() {
        assert_eq!(check_who("WHO"), true);
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn verify_max_clients() {
        // given how the clients array it is initialized,
        // its size cannot be larger than 32
        assert!(MAX_CLIENTS < 32);
    }

    #[test]
    fn verify_first_word() {
        let my_string = String::from("Hello World!");
        let word = first_word(&my_string);
        assert_eq!("Hello", word);
    }

    #[test]
    fn verify_first_word_with_initial_space() {
        let my_string = String::from("    Hello World!");
        let word = first_word(&my_string);
        assert_eq!("Hello", word);
    }

    #[test]
    fn verify_first_word_with_initial_tab() {
        let my_string = String::from("\tHello World!");
        let word = first_word(&my_string);
        assert_eq!("Hello", word);
    }

    #[test]
    fn verify_first_2_words_with_initial_space() {
        let my_string = String::from("    Hello    World!");
        let (word1, word2) = first_2_words(&my_string);
        assert_eq!(Some("Hello"), word1);
        assert_eq!(Some("World!"), word2);
    }

    #[test]
    fn verify_first_2_words_with_initial_space_2() {
        let my_string = String::from("\tHello\tWorld!");
        let (word1, word2) = first_2_words(&my_string);
        assert_eq!(Some("Hello"), word1);
        assert_eq!(Some("World!"), word2);
    }

    #[test]
    fn verify_first_2_words_with_initial_space_3() {
        let my_string = String::from(" \t Hello \t World!");
        let (word1, word2) = first_2_words(&my_string);
        assert_eq!(Some("Hello"), word1);
        assert_eq!(Some("World!"), word2);
    }
}
