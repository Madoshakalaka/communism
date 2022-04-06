use aws_sdk_ec2::model::InstanceStateName;
use bincode::{Decode, Encode};


#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ServerStatus {
    pub host: String,
    pub container: ContainerStatus,
    pub online: OnlinePeople,
}


#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode)]
pub enum ClientOpt {
    /// Power on the server.
    On,
    /// Power off the server.
    Off,
    /// Reboot the server.
    Reboot,
}



// reference output of `sudo docker-compose up -d`
//
// CONTAINER ID   IMAGE                   COMMAND    CREATED       STATUS                            PORTS                                                      NAMES
// 93b4bc8169e5   itzg/minecraft-server   "/start"   3 hours ago   Up 6 seconds (health: starting)   0.0.0.0:25565->25565/tcp, :::25565->25565/tcp, 25575/tcp   mc_mc_1

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum ContainerStatus{
    Unknown,
    NotUp,
    Up
}


#[derive(Encode, Decode, PartialEq, Debug)]
pub enum OnlinePeople{
    Unknown,
    Known(String)
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
