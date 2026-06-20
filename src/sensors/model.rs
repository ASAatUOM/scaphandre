use std::error::Error;
use crate::sensors::{CPUSocket, Domain, Record, RecordReader, Sensor, Topology};

pub struct ModelSensor{

}

impl ModelSensor{
    pub fn new(
        buffer_per_socket_max_kbytes: u16,
        buffer_per_domain_max_kbytes: u16,
        virtual_machine: bool,
    ) -> ModelSensor{
        ModelSensor{}
    }
}

impl Sensor for ModelSensor{
    fn get_topology(&self) -> Box<Option<Topology>> {
        panic!()
    }

    fn generate_topology(&self) -> Result<Topology, Box<dyn Error>> {
        todo!()
    }
}

impl RecordReader for Topology {
    fn read_record(&self) -> Result<Record, Box<dyn Error>> {
        todo!()
    }

}

impl RecordReader for CPUSocket{
    fn read_record(&self) -> Result<Record, Box<dyn Error>> {
        todo!()
    }
}

impl RecordReader for Domain{
    fn read_record(&self) -> Result<Record, Box<dyn Error>> {
        todo!()
    }
}