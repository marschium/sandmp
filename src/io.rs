use std::time::{Instant, Duration};
use std::sync::mpsc::{Receiver, Sender};
use std::net::{UdpSocket, SocketAddr};

use serde::{Deserialize, Serialize};
use rmps::{Deserializer, Serializer};

//use super::game::World;
//use super::particles::BLOCK_SIZE;

#[derive(Serialize, Deserialize, Debug)]
pub enum Msg{
    NewClient,
    TextureUpdate{
        x: i32,
        y: i32,
        data: Vec::<u8>
    }
}

// TODO move back into server code and keep common IO msg here?
pub struct InboundMessages {
    msg_in_sender: Sender<(Msg, SocketAddr)>,
    socket: UdpSocket
}

impl InboundMessages {

    pub fn new(msg_in_sender: Sender<(Msg, SocketAddr)>) -> InboundMessages {
        let socket = UdpSocket::bind("0.0.0.0:34254").unwrap();
        InboundMessages {
            msg_in_sender,
            socket
        }
    }

    pub fn start_listening(&self){
        loop {
            let mut buf = [0; 512];
            match self.socket.recv_from(&mut buf) {
                Ok((buf_size, src_addr)) => {
                    let msg: Msg = rmp_serde::from_read_ref(&buf[..]).unwrap();
                    let data = buf[..buf_size].to_vec();
                    self.msg_in_sender.send((msg, src_addr));
                },
                Err(_) => {

                }
            }
        }        
    }
}

pub struct OutboundMessages {
    socket: UdpSocket
} 

impl OutboundMessages {

    pub fn new() -> OutboundMessages {
        let socket = UdpSocket::bind("0.0.0.0:34255").unwrap();
        OutboundMessages {
            socket
        }
    }


    pub fn send_world(&self, clients: &Vec::<SocketAddr>, world: &World) {
        let s = Instant::now();
        let mut bufs = Vec::new();
        for (_, block) in world.all_blocks() {
            let mut buf = Vec::new();
            if block.updated {
                rmp::encode::write_i32(&mut buf, block.get_pos().0);
                rmp::encode::write_i32(&mut buf, block.get_pos().1);
                rmp::encode::write_bin_len(&mut buf, block.get_texture().len() as u32);
                rmp::encode::write_bin(&mut buf, block.get_texture());
                rmp::encode::write_i32(&mut buf, 69);
                bufs.push(buf);
            }        
        }  
        for client in clients {
            for buf in bufs.iter() {
                self.socket.send_to(&buf, client);
            }
        }  
         
        debug!("Serialize and send {:?}micros", s.elapsed().as_micros());
    }
}