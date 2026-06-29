use std::error::Error;
use std::collections::HashMap;
use crate::sensors::{units, CPUSocket, Domain, Model, Record, RecordReader, Sensor, Topology};
use crate::sensors::utils::current_system_time_since_epoch;

pub struct ModelSensor{
}

impl ModelSensor{
    pub fn new(
        _buffer_per_socket_max_kbytes: u16,
        _buffer_per_domain_max_kbytes: u16,
        _virtual_machine: bool,
    ) -> ModelSensor{
        ModelSensor{
        }
    }
}

impl Sensor for ModelSensor{
    fn get_topology(&self) -> Box<Option<Topology>> {
        let sensor_data = HashMap::new();
        let mut topology = Topology::new(sensor_data);

        let file = match std::fs::File::open("model.json"){
            Ok(file) => file,
            Err(_error) => {
                return Box::new(None);
            }
        };

        let reader = std::io::BufReader::new(file);
        let model: Model = match serde_json::from_reader(reader){
            Ok(model) => model,
            Err(_error) => {
                return Box::new(None);
            }
        };
        topology.model = model;
        Box::new(Some(topology))
    }

    fn generate_topology(&self) -> Result<Topology, Box<dyn Error>> {
        let topology = self.get_topology();
        match *topology {
            Some(t) => {
                Ok(t)
            }
            None => {
                Err("failed to generate topology".into())
            }
        }
    }
}

impl RecordReader for Topology {
    fn read_record(&self) -> Result<Record, Box<dyn Error>> {


        let mut tot : f64 = 0f64;
        // take sum of terms
        for term in self.model.terms.iter(){

            // read file
            let content = match std::fs::read_to_string(&term.path){
                Ok(content) => content,
                Err(_err)=>{
                    panic!("could not read term's file");
                }
            };

            let words: Vec<&str> = content.split_whitespace().collect(); // since some terms may read

            for i in 0..term.coefficient.len() {
                let word = match words.get(term.word_no[i]) {
                    Some(word) => word,
                    None => panic!("Could not read the {}'th word", term.word_no[i]),
                };
                let n: f64 = match word.parse() {
                    Ok(n) => n,
                    Err(_) => panic!("Could not parse word '{}' as f64", word),
                };
                tot += n.powf(term.power[i]) * term.coefficient[i];
            }

        }

        Ok(Record::new(
            current_system_time_since_epoch(),
            tot.to_string(),
            units::Unit::MicroJoule
        ))
    }

}

// TODO see what to do with these, currently not needed
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