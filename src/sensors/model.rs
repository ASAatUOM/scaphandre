use crate::sensors::utils::current_system_time_since_epoch;
use crate::sensors::{units, CPUSocket, Domain, Model, Record, RecordReader, Sensor, Topology};
use std::cell::Cell;
use std::collections::HashMap;
use std::error::Error;

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
        // open file
        let file = match std::fs::File::open("model.json"){
            Ok(file) => file,
            Err(_error) => {
                return Box::new(None);
            }
        };
        // read model file
        let reader = std::io::BufReader::new(file);
        let mut model: Model = match serde_json::from_reader(reader){
            Ok(model) => model,
            Err(_error) => {
                return Box::new(None);
            }
        };

        // read initial values
        for fterm in model.file_terms.iter_mut(){
            fterm.terms.sort_by_key(|term|term.word_no); // sort to be able to use .split_whitespace() iterator

            // read file
            let content = match std::fs::read_to_string(&fterm.path){
                Ok(content) => content,
                Err(_err)=>{
                    panic!("could not read term's file");
                }
            };

            let mut words = content.split_whitespace();
            let mut current = 0;
            // init last_read_value field of terms
            for term in fterm.terms.iter_mut(){

                match words.nth(term.word_no - current) {
                    None => {
                        panic!("Could not read word {} in {}", term.word_no, fterm.path);
                    }
                    Some(word) => {
                        term.last_read_value = match word.parse::<f64>() {
                            Ok(val) => { Cell::from(val) }
                            Err(_) => {
                                panic!("Could not parse word {} in {}", term.word_no, fterm.path);
                            }
                        };
                    }
                };
                current = term.word_no + 1;
            }

        }
        model.last_read_value.set(0);
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

// TODO see if calling RecordReader irregularly (varying time intervals) messes with linear or polynomial models
impl RecordReader for Topology {
    fn read_record(&self) -> Result<Record, Box<dyn Error>> {

        let mut tot : f64 = 0f64;

        // read initial values
        for fterm in self.model.file_terms.iter(){

            // read file
            let content = match std::fs::read_to_string(&fterm.path){
                Ok(content) => content,
                Err(_err)=>{
                    panic!("could not read term's file");
                }
            };

            let mut words = content.split_whitespace();
            let mut current = 0;
            // init last_read_value field of terms
            for term in fterm.terms.iter(){
                // update term.last_read_value
                let prev = term.last_read_value.get();
                let new_val:f64 = match words.nth(term.word_no - current) {
                    None => {
                        panic!("Could not read word {} in {}", term.word_no, fterm.path);
                    }
                    Some(word) => {
                        match word.parse::<f64>() {
                            Ok(val) => { val }
                            Err(_) => {
                                panic!("Could not parse word {} in {}", term.word_no, fterm.path);
                            }
                        }
                    }
                };
                // adding term value to total
                tot += term.coefficient*((new_val-prev).powf(term.power));
                // updating last read value
                term.last_read_value.set(new_val);
                current = term.word_no + 1;
            }

        }
        let result = (tot as i128)+self.model.last_read_value.get();
        self.model.last_read_value.set(result);
        Ok(Record::new(
            current_system_time_since_epoch(),
            result.to_string(),// Note: scaphandre expects the record value to be a digit
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