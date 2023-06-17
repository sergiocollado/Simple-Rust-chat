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

use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::env;
use std::process;
use std::sync::{Arc, Mutex};
use std::str;

const MAX_CLIENTS: usize = 20; // max clients cannot be >32, because the way the array initialization is done
const MAX_NAME_LEN: usize = 20;
const MAX_MESSAGE_SIZE: usize = 512;
const MAX_HOSTNAME_SIZE: usize = 50;
const VERSION: &str = "Simple Rust Chat Server v0.1\n";
// https://www.sitepoint.com/rust-global-variables/
// https://www.howtosolutions.net/2022/12/rust-create-global-variable-mutable-struct-without-unsafe-code-block/

type ClientsStreamArray = Arc<Mutex<[Option<TcpStream>; MAX_CLIENTS]>>;
type ClientsNameArray = Arc<Mutex<[Option<[u8; MAX_NAME_LEN]>; MAX_CLIENTS]>>;

fn main() {

    let args: Vec<String> = env::args().collect();

    verify_arguments(&args);

    let port = &args[1];
    println!("port: {}", port);

    // initialize array of clients
    // ref: https://www.joshmcguigan.com/blog/array-initialization-rust/
    // The primary downside to this method is it only works for arrays up to size 32.
    assert!(MAX_CLIENTS < 32);

    let clients_streams : ClientsStreamArray = Arc::new(Mutex::new(Default::default()));
    let clients_names : ClientsNameArray = Arc::new(Mutex::new([None; MAX_CLIENTS]));

    // create a listening socket
    let listener = TcpListener::bind("0.0.0.0:".to_owned() + port).expect("Error: Bind failed!");

    loop { // you could do the same without a loop with `listener.incomming()`.
        match listener.accept() {                    // new connection accepted
            Ok((stream, addr)) => {
                println!("New connection accepted: :{:?}, {:?}", stream, addr);

                for i in 0..MAX_CLIENTS {
                    println!("search loop: {}", i);

                   // if clients_streams[i].is_none() // this is just for debugging. TODO: remove
                   // {
                   //     println!("Debug log: vector clients: {:?}", clients_streams);
                   // }

                    if clients_streams.lock().unwrap()[i].is_none() {
                        println!("New client: pos({}): {:?}", i, addr);

                        //include/update this stream, in the array of clientsStreams
                        {
                            clients_streams.lock().unwrap()[i]
                                = Some(stream.try_clone().expect("failure trying to clone a stream"));
                        }

                        let client_names_array = Arc::clone(&clients_names);
                        //let client_stream_array = Arc::clone(&clients_streams);

                        thread::spawn(move || {
                            // connection suceeded
                            handle_client(stream, i, client_names_array); //, client_stream_array);
                        });

                        break; //once the new connection is registered, end the loop.
                    }

                }
                println!("vector clients after the loop on clients: {:?}", clients_streams);
            },
            Err(error) => {
                println!("Error: couldn't get client {error:?}");
                break
            },
        }
    }

    // close the socket server;
    drop(listener);
}

fn handle_client(mut stream: TcpStream, index: usize,
                 clients_array: ClientsNameArray)  {
                 //_stream_array: ClientsStreamArray) {
    let mut data = [0 as u8; MAX_MESSAGE_SIZE]; // using 512 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {

            println!("INPUT DATA: {:?}", data); // TODO: remove, just for debugging

            let clients1 = Arc::clone(&clients_array);
            server_chat_output(&data, index, size, clients1);

            let clients = Arc::clone(&clients_array);
            handle_commands(&data, index, clients);

            //TODO: check the borrowing of those clients!

            data = [0; MAX_MESSAGE_SIZE]; // clean the buffer for the next inter
            true
        },
        Err(_) => {
            println!("An error ocurred, terminating connection with {:?}", stream); stream.shutdown(Shutdown::Both).unwrap();
            //TODO: this means the socket was broken the client must be removed from the lists.
            false
        }
    } {}
}

fn handle_commands(input: &[u8], index: usize, clients_array: ClientsNameArray )
{
    let str_input = str::from_utf8(input).unwrap();

    if check_join_u8(input) {

        println!("JOIN command detected");
        let _indice : usize = index;               // TODO: simplify name
        let _clients = Arc::clone(&clients_array);
        handle_join(input, _indice, _clients);

    } else if check_who(str_input) {

        handle_who(&clients_array);
    }
}

fn broadcast(_message: &str, _index: usize, _clients_streams : &ClientsStreamArray)
{

}

fn verify_arguments(args: &Vec<String>) {

    println!("arguments: {:?}", args);

    if args.len() == 0 {
        println!("usage is: ./tcp_server <port number> \ni.e.: ./tcp_server 1153");
        process::exit(1);
    }

    if args.len() > 2 {
        println!("usage is: ./tcp_server <port number> \ni.e.: ./tcp_server 1153");
        process::exit(1);
    }
}

fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();

    std::str::from_utf8(first_word_u8(&bytes))
        .expect("fn first_word: wrong conversion u8 -> str")
}

fn first_word_u8(s: &[u8]) -> &[u8] {
    let mut i1 : usize = 0;

    for (i, &item) in s.iter().enumerate() {
        if !(item == b' ' || item == b'\t' || item == b'\n' || item == b'\r')
        {
            i1 = i;
            break;
        }
    }

    let s2 = &s[i1..];

    for (i, &item) in s2.iter().enumerate() {
        if item == b' ' || item == b'\t' || item == b'\n' || item == b'\r'
        {
            let i2 = i1 + i;
            std::str::from_utf8(&s[i1..i2]).unwrap();
            return &s[i1..i2];
        }
    }

    &s[..]
}

fn first_2_words(s: &str) -> (Option<&str>, Option<&str>) {
    let mut iter = s.split_ascii_whitespace();
    let word1 = iter.next();
    let word2 = iter.next();
    (word1, word2)
}

fn check_command(command: &str, input: &str) -> bool {
    let first_word = first_word(input);
    command.trim() == first_word
}

fn check_command_u8(command: &str, input: &[u8]) -> bool {
    let first_word_u8 = first_word_u8(input);
    command == std::str::from_utf8(first_word_u8).unwrap()
}

//fn check_join(input: &str) -> bool {
//    check_command("JOIN", input)
//}

fn check_join_u8(input: &[u8]) -> bool {
    check_command_u8("JOIN", input)
}

fn handle_join(input: &[u8], index: usize, clients_array: ClientsNameArray) {
    println!("Detected JOIN command {input:?}"); // TODO: remove just for debugging

    // TODO: add check to verify the user has not JOINed previosly
    let (_, name) = first_2_words(std::str::from_utf8(input).unwrap());
    if name.is_some()
    {
        let name = name.unwrap();

        let mut client_name : [u8; MAX_NAME_LEN] = [0; MAX_NAME_LEN];
        let mut i: usize = 0;
        while i < MAX_NAME_LEN && i < name.as_bytes().len()
        {
            client_name[i] = name.as_bytes()[i];
            i = i + 1;
        }

        {
            let mut array_clients = clients_array.lock().unwrap();
            array_clients[index] = Some(client_name);
        }

        println!("{} wants to join to the chat", name);
        // TODO broadcast new join to the clients
    }
}

fn server_chat_output(input: &[u8], index: usize, size: usize, clients_array: ClientsNameArray)
{
    // output in stdout
    let name_array_clone = Arc::clone(&clients_array);
    let name_arrays_mutex = name_array_clone.lock().unwrap();
    let name_arrays : [Option<[u8; MAX_NAME_LEN]>; MAX_CLIENTS] = *name_arrays_mutex;
    let name_i : Option<[u8; MAX_NAME_LEN]> = name_arrays[index];
    if name_i.is_some()
    {
        let name = name_i.unwrap();
        print!("[{}]", str::from_utf8(&name).unwrap().to_string().trim_matches(char::from(0)));
    }

    std::io::stdout().write_all(&input[0..size]).expect("Error writing to stdout");
}

fn check_who(input: &str) -> bool {
    check_command("WHO", input)
}

fn handle_who(clients_array: &ClientsNameArray) {

    let name_array_clone = Arc::clone(clients_array);
    let name_arrays_mutex = name_array_clone.lock().unwrap();
    let name_arrays : [Option<[u8; MAX_NAME_LEN]>; MAX_CLIENTS] = *name_arrays_mutex;
    for name in name_arrays {
        if name.is_some() {
            println!("{}", str::from_utf8(&name.unwrap()).unwrap().to_string().trim_matches(char::from(0)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(Some("Hello") , word1);
        assert_eq!(Some("World!"), word2);
    }

    #[test]
    fn verify_first_2_words_with_initial_space_2() {
        let my_string = String::from("\tHello\tWorld!");
        let (word1, word2) = first_2_words(&my_string);
        assert_eq!(Some("Hello") , word1);
        assert_eq!(Some("World!"), word2);
    }

    #[test]
    fn verify_first_2_words_with_initial_space_3() {
        let my_string = String::from(" \t Hello \t World!");
        let (word1, word2) = first_2_words(&my_string);
        assert_eq!(Some("Hello") , word1);
        assert_eq!(Some("World!"), word2);
    }

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
}
