#[derive(Debug)]
pub struct AveragedCollection {
    list: Vec<i32>,
    average: f64,
}

impl AveragedCollection {
    
    pub fn new() -> Self {
       Self {
        list: Vec::new(),
        average: 0.,
       }
    }

    pub fn add(&mut self, element: i32) {
        self.list.push(element);
        self.update_average();
    }


    pub fn remove(&mut self) -> Option<i32> {
        let result = self.list.pop();
        match result {
            Some(_) => { 
                self.update_average();
                result
            },
            None => None,
        }
    }

    pub fn average(&self) -> f64 {
        self.average
    }

    fn update_average(&mut self) {
        let sum: i32 = self.list.iter().sum();
        self.average = if self.list.is_empty() {
            0.
        } else {
            sum as f64  / self.list.len() as f64
        } 
    }
}

