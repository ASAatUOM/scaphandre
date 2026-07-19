use crate::sensors::utils::current_system_time_since_epoch;
use crate::sensors::{units, CPUSocket, Domain, Model, Record, RecordReader, Sensor, Topology};
use std::collections::HashMap;
use std::error::Error;

pub struct ModelSensor{
    use_polynomial: bool,
    model_file: Option<String>
}

impl ModelSensor{
    pub fn new(
        _buffer_per_socket_max_kbytes: u16,
        _buffer_per_domain_max_kbytes: u16,
        _virtual_machine: bool,
        model_file: Option<String>,
        use_poly: bool
    ) -> ModelSensor{
        ModelSensor{
            use_polynomial: use_poly,
            model_file
        }
    }


}

// returns (cbusy,ctotal)
fn get_cpu_utilisation()->(u64, u64) {
    let contents = std::fs::read_to_string("/proc/stat").expect("Error reading /proc/stat");
    let mut words = contents.split_whitespace().take(5);
    words.next();
    let cuser = words.next().unwrap().parse::<u64>().unwrap();
    let cnice = words.next().unwrap().parse::<u64>().unwrap();
    let csystem = words.next().unwrap().parse::<u64>().unwrap();
    let cidle = words.next().unwrap().parse::<u64>().unwrap();
    let cbusy = cuser+cnice+csystem;
    (cbusy,cbusy+cidle)
}

impl Sensor for ModelSensor{
    fn get_topology(&self) -> Box<Option<Topology>> {
        let sensor_data = HashMap::new();
        let mut topology = Topology::new(sensor_data);

        // model file path
        let path = self
            .model_file
            .as_deref()
            .unwrap_or("RaspberryPiModel.json");

        // open file
        let file = match std::fs::File::open(path){
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

        model.use_linear = !self.use_polynomial;
        model.last_reading.set(current_system_time_since_epoch().as_secs_f64());

        // read initial values
        let cval = get_cpu_utilisation();
        model.last_cbusy.set(cval.0);
        model.last_ctot.set(cval.1);

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

        let mut output: f64 = 0f64;
        let cutil = get_cpu_utilisation();
        // calculate cpu utilisation
        let util: f64 = ((cutil.0-self.model.last_cbusy.get()) as f64) /((cutil.1-self.model.last_ctot.get()) as f64);
        let current_time = current_system_time_since_epoch();
        // update values
        self.model.last_cbusy.set(cutil.0);
        self.model.last_ctot.set(cutil.1);

        if self.model.use_linear {
            // use linear model
            let lin = &self.model.linear;
            output += lin.u*util + lin.c;
        }else{
            // use polynomial model
            output += self.model.polynomial.intercept;
            for i in 0..self.model.polynomial.coefficients.len(){
                output += self.model.polynomial.coefficients[i]*util.powi(i as i32);
            }

        }
        let tot = &self.model.total;
        output *= current_time.as_secs_f64()-self.model.last_reading.get(); // scaling according to time
        output *= 1000000.0; // converting to MicroJoule to minimize rounding error
        tot.set(tot.get()+output);// updating last reading time

        self.model.last_reading.set(current_time.as_secs_f64());

        Ok(Record::new(
            current_time,
            (tot.get() as u64).to_string(), // Note: scaphandre expects the record value to be a digit
            units::Unit::MicroJoule
        ))
    }

}

// TODO see what to do with these
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